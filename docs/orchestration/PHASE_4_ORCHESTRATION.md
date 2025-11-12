# Phase 4: Autonomous Code Orchestration Engine

## Overview

Execute coding tasks automatically via Claude Code with verification, auto-retry, and progress tracking. This is where code gets generated autonomously from the task queue.

## Prerequisites

- ✅ Phase 1: CodebaseContext available
- ✅ Phase 2: TaskQueue with dependency resolution
- ✅ Phase 3: Tasks generated with rich prompts
- ✅ Existing Claude Code `/meeting` command integration

## Goals

1. **Autonomous Execution** - Pull tasks from queue and execute without human intervention
2. **Verification** - Compile and test after each task
3. **Auto-Retry** - Retry failed tasks with error context (up to 3 attempts)
4. **Progress Tracking** - Real-time status updates
5. **Git Integration** - Commit working code after successful tasks

## Success Criteria

- ✅ 80%+ tasks complete successfully on first attempt
- ✅ 60%+ of failures fixed by auto-retry
- ✅ 100% of committed code compiles
- ✅ Execution doesn't hang on stuck tasks
- ✅ User sees real-time progress

## Orchestration Loop

```
┌─────────────────────────────────────────┐
│   Orchestrator Loop                     │
│                                         │
│   ┌──> Get next ready task             │
│   │    (dependencies met, status=Ready) │
│   │                                      │
│   │    Build execution prompt           │
│   │    (task + context + references)    │
│   │                                      │
│   │    Execute via Claude Code          │
│   │    (async, non-blocking)            │
│   │                                      │
│   │    Verify result                    │
│   │    (compile, test, lint)            │
│   │                                      │
│   │    Success?                         │
│   │    ├─ Yes: Mark complete            │
│   │    │       Git commit               │
│   │    │       Emit event                │
│   │    │                                 │
│   │    └─ No: Retry count < 3?         │
│   │          ├─ Yes: Retry with errors  │
│   │          │       (add error context) │
│   │          │                           │
│   │          └─ No: Mark failed         │
│   │                Emit event            │
│   │                                      │
│   └─── Loop until queue empty           │
│                                         │
└─────────────────────────────────────────┘
```

## Core Implementation

### File: `src-tauri/src/orchestration/orchestrator.rs`

```rust
pub struct CodeOrchestrator {
    meeting_id: String,
    task_queue: Arc<Mutex<TaskQueue>>,
    codebase_context: CodebaseContext,
    claude_code_client: ClaudeCodeClient,
    verifier: CodeVerifier,
    is_running: Arc<AtomicBool>,
}

impl CodeOrchestrator {
    pub async fn start(&self) -> Result<(), OrchestrationError> {
        info!("Starting orchestration for meeting {}", self.meeting_id);
        self.is_running.store(true, Ordering::SeqCst);

        while self.is_running.load(Ordering::SeqCst) {
            // Get next ready task
            let task = {
                let queue = self.task_queue.lock().await;
                queue.get_next_ready_task().cloned()
            };

            match task {
                Some(task) => {
                    info!("Executing task: {}", task.title);
                    self.execute_task(task).await?;
                }
                None => {
                    // No ready tasks, check if we're done
                    let queue = self.task_queue.lock().await;
                    let all_tasks = queue.get_all_tasks();

                    let has_pending = all_tasks.iter()
                        .any(|t| matches!(t.status, TaskStatus::Pending | TaskStatus::Ready));

                    if !has_pending {
                        info!("All tasks complete!");
                        break;
                    }

                    // Wait a bit before checking again
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        Ok(())
    }

    async fn execute_task(&self, mut task: CodingTask) -> Result<(), OrchestrationError> {
        // Update status to InProgress
        {
            let mut queue = self.task_queue.lock().await;
            queue.update_task_status(&task.id, TaskStatus::InProgress)?;
        }

        // Try up to 3 times
        for attempt in 1..=3 {
            let result = self.execute_task_attempt(&task, attempt).await;

            match result {
                Ok(verification) if verification.success => {
                    // Success! Mark complete and commit
                    {
                        let mut queue = self.task_queue.lock().await;
                        queue.update_task_status(&task.id, TaskStatus::Completed)?;
                    }

                    self.commit_changes(&task).await?;
                    info!("Task completed: {}", task.title);
                    return Ok(());
                }
                Ok(verification) => {
                    // Verification failed, add error context for retry
                    warn!("Task verification failed (attempt {}): {}", attempt, verification.error_message);

                    if attempt < 3 {
                        // Add error context to task for next attempt
                        task.prompt = self.add_error_context(&task, &verification);
                    }
                }
                Err(e) => {
                    error!("Task execution error (attempt {}): {}", attempt, e);

                    if attempt == 3 {
                        // Final failure
                        let mut queue = self.task_queue.lock().await;
                        queue.update_task_status(&task.id, TaskStatus::Failed)?;
                        return Err(e);
                    }
                }
            }
        }

        // All attempts failed
        let mut queue = self.task_queue.lock().await;
        queue.update_task_status(&task.id, TaskStatus::Failed)?;

        Ok(())
    }

    async fn execute_task_attempt(
        &self,
        task: &CodingTask,
        attempt: usize,
    ) -> Result<VerificationResult, OrchestrationError> {
        let attempt_start = chrono::Utc::now();

        // Build execution prompt with full context
        let prompt = self.build_execution_prompt(task, attempt);

        // Execute via Claude Code
        let result = self.claude_code_client
            .execute_meeting_command(&prompt)
            .await?;

        // Verify the result
        let verification = self.verifier.verify(&result, task).await?;

        // Record attempt
        let mut queue = self.task_queue.lock().await;
        if let Some(task_ref) = queue.get_task_mut(&task.id) {
            task_ref.attempts.push(TaskAttempt {
                attempt_number: attempt,
                started_at: attempt_start.to_rfc3339(),
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
                result: if verification.success {
                    TaskResult::Success
                } else {
                    verification.error_type.clone()
                },
                error_message: verification.error_message.clone(),
                verification_output: Some(verification.output.clone()),
            });
        }

        Ok(verification)
    }

    fn build_execution_prompt(&self, task: &CodingTask, attempt: usize) -> String {
        let mut prompt = task.prompt.clone();

        if attempt > 1 {
            prompt.push_str(&format!(
                "\n\nATTEMPT {}: Previous attempts failed. Review errors and try a different approach.\n",
                attempt
            ));
        }

        // Add project context
        prompt.push_str(&format!(
            "\n\nPROJECT CONTEXT:\n- Path: {}\n- Tech Stack: {} with {}\n",
            self.codebase_context.project_path,
            self.codebase_context.tech_stack.primary_language,
            self.codebase_context.tech_stack.frameworks.join(", ")
        ));

        prompt
    }

    fn add_error_context(&self, task: &CodingTask, verification: &VerificationResult) -> String {
        format!(
            "{}\n\nPREVIOUS ATTEMPT FAILED:\nError Type: {:?}\n\nError Output:\n{}\n\nPlease fix these errors and try again.\n",
            task.prompt,
            verification.error_type,
            verification.error_message
        )
    }

    async fn commit_changes(&self, task: &CodingTask) -> Result<(), OrchestrationError> {
        // Use existing git integration
        let commit_message = format!(
            "{}\n\nAuto-generated from task: {}\n\n🤖 Generated with MeetingCoder Orchestration",
            task.title,
            task.id
        );

        // Execute git commands
        Command::new("git")
            .args(&["add", "."])
            .current_dir(&self.codebase_context.project_path)
            .output()
            .await?;

        Command::new("git")
            .args(&["commit", "-m", &commit_message])
            .current_dir(&self.codebase_context.project_path)
            .output()
            .await?;

        info!("Committed changes for task: {}", task.title);
        Ok(())
    }
}
```

### Code Verifier

**File: `src-tauri/src/orchestration/verifier.rs`**

```rust
pub struct CodeVerifier {
    project_path: PathBuf,
    tech_stack: TechStack,
}

pub struct VerificationResult {
    pub success: bool,
    pub error_type: TaskResult,
    pub error_message: String,
    pub output: String,
}

impl CodeVerifier {
    pub async fn verify(
        &self,
        execution_result: &str,
        task: &CodingTask,
    ) -> Result<VerificationResult, VerificationError> {
        // 1. Check if files were created/modified
        let files_exist = self.verify_files_exist(task).await?;
        if !files_exist {
            return Ok(VerificationResult {
                success: false,
                error_type: TaskResult::Unknown,
                error_message: "Expected files were not created".to_string(),
                output: "".to_string(),
            });
        }

        // 2. Compile check
        let compile_result = self.verify_compilation().await?;
        if !compile_result.success {
            return Ok(compile_result);
        }

        // 3. Run tests (if applicable)
        let test_result = self.verify_tests().await?;
        if !test_result.success {
            return Ok(test_result);
        }

        // All checks passed
        Ok(VerificationResult {
            success: true,
            error_type: TaskResult::Success,
            error_message: String::new(),
            output: "All verification checks passed".to_string(),
        })
    }

    async fn verify_compilation(&self) -> Result<VerificationResult, VerificationError> {
        // Run appropriate build command based on tech stack
        let build_command = match self.tech_stack.build_tool.as_deref() {
            Some("Vite") => vec!["bun", "run", "build"],
            Some("Cargo") => vec!["cargo", "build"],
            _ => vec!["npm", "run", "build"],
        };

        let output = Command::new(build_command[0])
            .args(&build_command[1..])
            .current_dir(&self.project_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(VerificationResult {
                success: true,
                error_type: TaskResult::Success,
                error_message: String::new(),
                output: String::from_utf8_lossy(&output.stdout).to_string(),
            })
        } else {
            Ok(VerificationResult {
                success: false,
                error_type: TaskResult::CompilationError,
                error_message: String::from_utf8_lossy(&output.stderr).to_string(),
                output: String::from_utf8_lossy(&output.stdout).to_string(),
            })
        }
    }

    async fn verify_tests(&self) -> Result<VerificationResult, VerificationError> {
        // Run appropriate test command
        let test_command = match self.tech_stack.primary_language.as_str() {
            "TypeScript" | "JavaScript" => vec!["bun", "test"],
            "Rust" => vec!["cargo", "test"],
            "Python" => vec!["pytest"],
            _ => return Ok(VerificationResult {
                success: true,  // Skip if unknown
                error_type: TaskResult::Success,
                error_message: String::new(),
                output: "Test verification skipped".to_string(),
            }),
        };

        let output = Command::new(test_command[0])
            .args(&test_command[1..])
            .current_dir(&self.project_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(VerificationResult {
                success: true,
                error_type: TaskResult::Success,
                error_message: String::new(),
                output: String::from_utf8_lossy(&output.stdout).to_string(),
            })
        } else {
            Ok(VerificationResult {
                success: false,
                error_type: TaskResult::TestFailure,
                error_message: String::from_utf8_lossy(&output.stderr).to_string(),
                output: String::from_utf8_lossy(&output.stdout).to_string(),
            })
        }
    }
}
```

### Claude Code Client

**File: `src-tauri/src/orchestration/claude_code_client.rs`**

```rust
pub struct ClaudeCodeClient {
    // Reuse existing Claude Code integration
}

impl ClaudeCodeClient {
    pub async fn execute_meeting_command(&self, prompt: &str) -> Result<String, ExecutionError> {
        // Call existing /meeting command integration
        // This leverages the existing Claude Code setup
        //
        // Implementation depends on how /meeting is currently integrated
        // Could be:
        // 1. Shell command execution
        // 2. API call to Claude Code service
        // 3. Direct integration with Claude API

        // Example: Shell execution
        let output = Command::new("claude-code")
            .args(&["/meeting", prompt])
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
```

## Tauri Commands

```rust
#[tauri::command]
async fn start_orchestration(
    meeting_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Get required components
    let task_queue = state.task_queues.lock().await
        .get(&meeting_id)
        .ok_or("Task queue not found")?
        .clone();

    let codebase_context = state.codebase_contexts.lock().await
        .get(&meeting_id)
        .ok_or("Codebase context not found")?
        .clone();

    // Create orchestrator
    let orchestrator = CodeOrchestrator::new(
        meeting_id.clone(),
        Arc::new(Mutex::new(task_queue)),
        codebase_context,
    );

    // Start in background
    let app_handle = state.app_handle.clone();
    tokio::spawn(async move {
        match orchestrator.start().await {
            Ok(_) => {
                app_handle.emit_all("orchestration_complete", &meeting_id).ok();
            }
            Err(e) => {
                error!("Orchestration failed: {}", e);
                app_handle.emit_all("orchestration_error", e.to_string()).ok();
            }
        }
    });

    Ok(())
}

#[tauri::command]
async fn stop_orchestration(
    meeting_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Signal orchestrator to stop
    if let Some(orchestrator) = state.orchestrators.lock().await.get(&meeting_id) {
        orchestrator.stop();
    }
    Ok(())
}
```

## Frontend UI

```typescript
export const OrchestratorView: React.FC<{ meetingId: string }> = ({ meetingId }) => {
  const [isRunning, setIsRunning] = useState(false);
  const [progress, setProgress] = useState({ completed: 0, total: 0 });

  const handleStart = async () => {
    setIsRunning(true);
    await invoke("start_orchestration", { meetingId });
  };

  const handleStop = async () => {
    await invoke("stop_orchestration", { meetingId });
    setIsRunning(false);
  };

  useEffect(() => {
    const listeners = [
      listen("task_status_updated", () => {
        // Refresh progress
        invoke<Task[]>("get_task_queue", { meetingId }).then(tasks => {
          setProgress({
            completed: tasks.filter(t => t.status === "completed").length,
            total: tasks.length,
          });
        });
      }),

      listen("orchestration_complete", () => {
        setIsRunning(false);
        toast.success("All tasks completed!");
      }),

      listen("orchestration_error", (event) => {
        setIsRunning(false);
        toast.error(`Orchestration error: ${event.payload}`);
      }),
    ];

    return () => {
      listeners.forEach(l => l.then(fn => fn()));
    };
  }, [meetingId]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2>Code Orchestration</h2>
        {!isRunning ? (
          <Button onClick={handleStart}>Start Orchestration</Button>
        ) : (
          <Button onClick={handleStop} variant="destructive">Stop</Button>
        )}
      </div>

      {isRunning && (
        <div className="space-y-2">
          <Progress value={(progress.completed / progress.total) * 100} />
          <p className="text-sm text-gray-600">
            {progress.completed} / {progress.total} tasks completed
          </p>
        </div>
      )}
    </div>
  );
};
```

## Testing Requirements

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_task_execution_success() {
        let orchestrator = create_test_orchestrator();
        let task = create_mock_task();

        let result = orchestrator.execute_task(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auto_retry_on_failure() {
        let orchestrator = create_test_orchestrator_with_failing_verifier();
        let task = create_mock_task();

        orchestrator.execute_task(task.clone()).await.ok();

        // Check that 3 attempts were made
        assert_eq!(task.attempts.len(), 3);
    }

    #[tokio::test]
    async fn test_verification_compilation_check() {
        let verifier = CodeVerifier::new(test_project_path(), test_tech_stack());
        let result = verifier.verify_compilation().await.unwrap();

        assert!(result.success);
    }
}
```

## File Structure

```
src-tauri/src/
├── orchestration/
│   ├── mod.rs                    # Module exports
│   ├── orchestrator.rs           # Main loop (300 lines)
│   ├── verifier.rs               # Verification (250 lines)
│   ├── claude_code_client.rs    # Execution client (150 lines)
│   └── types.rs                  # Data structures (100 lines)
└── commands/
    └── orchestration.rs          # Tauri commands (150 lines)

src/components/meeting/
└── OrchestratorView.tsx          # UI component (200 lines)
```

## Implementation Timeline

**Days 1-2:** Orchestrator loop and state management
**Days 3-4:** Code verifier with compile/test checks
**Days 5-6:** Auto-retry logic with error context
**Days 7-8:** Git integration and commit logic
**Days 9-10:** Frontend UI and testing

## Error Handling

```rust
pub enum OrchestrationError {
    TaskQueueError(String),
    ExecutionError(String),
    VerificationError(String),
    GitError(String),
    Timeout,
}

// Safety: Max execution time per task
const TASK_TIMEOUT: Duration = Duration::from_secs(300);  // 5 minutes

async fn execute_with_timeout(task: CodingTask) -> Result<()> {
    tokio::time::timeout(TASK_TIMEOUT, execute_task(task))
        .await
        .map_err(|_| OrchestrationError::Timeout)?
}
```

## Next Phase

Phase 5 adds intelligence layer: LLM code review, analytics, smart scheduling, and safety limits for production reliability.
