# Phase 3: Context-Aware Task Generation

## Overview

Automatically convert PRD user stories into concrete coding tasks with exact file locations, implementation suggestions, and dependency inference using codebase intelligence from Phase 1.

## Prerequisites

- ✅ Phase 1: CodebaseContext available
- ✅ Phase 2: TaskQueue ready to receive tasks
- ✅ PRD generation system (already implemented)

## Goals

1. **Automatic Conversion** - PRD finalized → Tasks generated automatically
2. **File Location Precision** - 90%+ of suggested paths are correct
3. **Context-Rich Prompts** - Include reference code and patterns
4. **Dependency Inference** - Automatically determine task order
5. **Fast Generation** - Complete in < 60 seconds for typical PRD

## Success Criteria

- ✅ Generates actionable tasks from 90%+ of PRD user stories
- ✅ File paths are correct 90%+ of the time
- ✅ Dependencies are inferred with 85%+ accuracy
- ✅ Generated prompts contain sufficient context for Claude Code
- ✅ Triggers automatically when PRD is finalized

## Trigger Conditions

Task generation happens when:
1. PRD is finalized (meeting end or manual generation)
2. CodebaseContext is available (Phase 1 complete)
3. No tasks already exist for this PRD version

## Core Process Flow

```
PRD Finalized Event
    ↓
Check: CodebaseContext ready?
    ↓ (yes)
TaskGenerator.generate_from_prd(prd, codebase_context)
    ↓
For each user story:
  ├─> LLM analyzes story + context
  ├─> Suggests implementation approach
  ├─> Determines affected files
  ├─> Finds similar code for reference
  └─> Infers dependencies
    ↓
Concrete CodingTasks created
    ↓
DependencyResolver resolves order
    ↓
TaskQueue populated
    ↓
Event: "tasks_ready" → UI updates
```

## LLM Prompt Structure

### System Prompt for Task Generation

```
You are a coding task generator. Your job is to convert PRD user stories into concrete, executable coding tasks.

PROJECT CONTEXT:
- Tech Stack: {{tech_stack}}
- Architecture: {{architecture_pattern}}
- Existing Features: {{features_list}}
- Conventions: {{naming_conventions}}

INTEGRATION POINTS:
{{integration_points_with_examples}}

TASK REQUIREMENTS:
1. Specify exact files to create/modify
2. Suggest implementation approach
3. Reference similar existing code
4. Identify dependencies on other tasks
5. Include enough context for autonomous execution

OUTPUT FORMAT:
{
  "tasks": [
    {
      "title": "Create ComponentName",
      "description": "Detailed explanation",
      "files_to_create": ["src/components/ComponentName.tsx"],
      "files_to_modify": ["src/components/index.ts"],
      "implementation_approach": "...",
      "reference_files": ["src/components/SimilarComponent.tsx"],
      "dependencies": ["task_id_1"],
      "priority": "high"
    }
  ]
}
```

### User Prompt for Each Story

```
USER STORY:
{{user_story.title}}
{{user_story.description}}

ACCEPTANCE CRITERIA:
{{acceptance_criteria}}

Generate concrete coding tasks to implement this user story.
Consider the project context and existing patterns.
```

## Implementation

### File: `src-tauri/src/task_generation/task_generator.rs`

```rust
pub struct TaskGenerator {
    llm_client: ClaudeClient,
}

impl TaskGenerator {
    pub async fn generate_from_prd(
        &self,
        prd: &PRDContent,
        codebase_context: &CodebaseContext,
    ) -> Result<Vec<CodingTask>, GenerationError> {
        let mut all_tasks = Vec::new();

        // Generate tasks for each user story
        for user_story in &prd.user_stories {
            let tasks = self.generate_tasks_for_story(
                user_story,
                codebase_context,
            ).await?;

            all_tasks.extend(tasks);
        }

        // Infer dependencies between tasks
        self.infer_dependencies(&mut all_tasks, codebase_context);

        // Validate and deduplicate
        self.validate_tasks(&all_tasks)?;

        Ok(all_tasks)
    }

    async fn generate_tasks_for_story(
        &self,
        user_story: &UserStory,
        context: &CodebaseContext,
    ) -> Result<Vec<CodingTask>, GenerationError> {
        // Build context-rich prompt
        let system_prompt = self.build_system_prompt(context);
        let user_prompt = self.build_user_prompt(user_story);

        // Call Claude API
        let response = self.llm_client.chat(
            &system_prompt,
            &user_prompt,
            ClaudeModel::Sonnet,
        ).await?;

        // Parse LLM response
        let task_specs: TaskGenerationResponse = serde_json::from_str(&response)?;

        // Convert to CodingTasks
        let tasks = task_specs.tasks.into_iter().map(|spec| {
            CodingTask {
                id: Uuid::new_v4().to_string(),
                title: spec.title,
                description: spec.description,
                priority: spec.priority,
                status: TaskStatus::Pending,
                files_to_create: spec.files_to_create,
                files_to_modify: spec.files_to_modify,
                prompt: self.build_execution_prompt(&spec, context),
                implementation_context: Some(ImplementationContext {
                    similar_files: spec.reference_files,
                    integration_point: self.find_integration_point(&spec, context),
                    tech_stack_notes: format!("Using {} with {}",
                        context.tech_stack.primary_language,
                        context.tech_stack.frameworks.join(", ")
                    ),
                    conventions: self.extract_conventions_for_task(&spec, context),
                }),
                dependencies: spec.dependencies,
                attempts: Vec::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            }
        }).collect();

        Ok(tasks)
    }

    fn build_execution_prompt(
        &self,
        task_spec: &TaskSpec,
        context: &CodebaseContext,
    ) -> String {
        format!(
            r#"
TASK: {}

DESCRIPTION:
{}

FILES TO CREATE:
{}

FILES TO MODIFY:
{}

IMPLEMENTATION APPROACH:
{}

REFERENCE IMPLEMENTATIONS:
{}

TECH STACK:
- Language: {}
- Frameworks: {}
- Styling: {}

CONVENTIONS:
{}

INTEGRATION:
{}

Please implement this task following the project's patterns and conventions.
"#,
            task_spec.title,
            task_spec.description,
            task_spec.files_to_create.join("\n"),
            task_spec.files_to_modify.join("\n"),
            task_spec.implementation_approach,
            self.format_reference_files(&task_spec.reference_files, context),
            context.tech_stack.primary_language,
            context.tech_stack.frameworks.join(", "),
            context.tech_stack.styling.as_ref().unwrap_or(&"N/A".to_string()),
            self.extract_conventions_for_task(task_spec, context),
            self.find_integration_point(task_spec, context),
        )
    }
}
```

### Dependency Inference

```rust
impl TaskGenerator {
    fn infer_dependencies(
        &self,
        tasks: &mut [CodingTask],
        context: &CodebaseContext,
    ) {
        // Build file dependency graph
        let mut file_to_task: HashMap<String, String> = HashMap::new();

        for task in tasks.iter() {
            for file in &task.files_to_create {
                file_to_task.insert(file.clone(), task.id.clone());
            }
        }

        // Infer dependencies based on file usage
        for task in tasks.iter_mut() {
            for file in &task.files_to_modify {
                // If this task modifies a file that another task creates,
                // this task depends on that task
                if let Some(creator_task_id) = file_to_task.get(file) {
                    if !task.dependencies.contains(creator_task_id) {
                        task.dependencies.push(creator_task_id.clone());
                    }
                }
            }

            // Infer dependencies from task titles/descriptions
            // e.g., "Add tests for UserAuth" depends on "Create UserAuth"
            for other_task in tasks.iter() {
                if task.id != other_task.id {
                    if self.is_likely_dependent(&task.title, &other_task.title) {
                        if !task.dependencies.contains(&other_task.id) {
                            task.dependencies.push(other_task.id.clone());
                        }
                    }
                }
            }
        }
    }

    fn is_likely_dependent(&self, task_title: &str, other_title: &str) -> bool {
        let task_lower = task_title.to_lowercase();
        let other_lower = other_title.to_lowercase();

        // "Add tests for X" depends on "Create X"
        if task_lower.contains("test") &&
           task_lower.contains(&other_lower.replace("create", "")) {
            return true;
        }

        // "Integrate X with Y" depends on "Create X" and "Create Y"
        if task_lower.contains("integrate") && other_lower.contains("create") {
            return true;
        }

        false
    }
}
```

### Reference Code Finding

```rust
impl TaskGenerator {
    fn format_reference_files(
        &self,
        reference_files: &[String],
        context: &CodebaseContext,
    ) -> String {
        reference_files.iter()
            .filter_map(|file_path| {
                // Read actual file content to include as reference
                std::fs::read_to_string(file_path).ok().map(|content| {
                    format!("// File: {}\n{}\n", file_path, content)
                })
            })
            .take(2)  // Limit to 2 reference files to avoid huge prompts
            .collect::<Vec<_>>()
            .join("\n---\n")
    }

    fn find_integration_point(
        &self,
        task_spec: &TaskSpec,
        context: &CodebaseContext,
    ) -> String {
        // Find relevant integration point based on task type
        for point in &context.integration_points {
            // Match based on files being modified
            if task_spec.files_to_modify.iter().any(|f| f.contains(&point.file_path)) {
                return format!(
                    "Register in {} at line {}\nExample:\n{}",
                    point.file_path,
                    point.line_number.unwrap_or(0),
                    point.example_code
                );
            }
        }

        "No specific integration point found".to_string()
    }
}
```

## Tauri Commands

```rust
#[tauri::command]
async fn generate_tasks_from_prd(
    meeting_id: String,
    prd_version: usize,
    state: State<'_, AppState>,
) -> Result<Vec<CodingTask>, String> {
    // 1. Get PRD
    let prd = state.prd_generator
        .get_prd_version(&meeting_id, prd_version)
        .ok_or("PRD not found")?;

    // 2. Get codebase context
    let codebase_context = state.codebase_contexts.lock().await
        .get(&meeting_id)
        .ok_or("Codebase context not available")?
        .clone();

    // 3. Generate tasks
    let generator = TaskGenerator::new(state.claude_client.clone());
    let tasks = generator.generate_from_prd(&prd, &codebase_context)
        .await
        .map_err(|e| e.to_string())?;

    // 4. Add to task queue
    let mut queues = state.task_queues.lock().await;
    let queue = queues.entry(meeting_id.clone())
        .or_insert_with(|| TaskQueue::new(&meeting_id));

    for task in &tasks {
        queue.add_task(task.clone())
            .map_err(|e| e.to_string())?;
    }

    // 5. Emit event
    state.app_handle.emit_all("tasks_generated", &tasks).ok();

    Ok(tasks)
}
```

## Auto-Triggering on PRD Finalization

### In PRD Generation Module

```rust
// In prd_generator.rs, after PRD is finalized:
async fn on_prd_finalized(
    &self,
    meeting_id: &str,
    prd_version: usize,
    app_state: &AppState,
) -> Result<()> {
    // Check if codebase context is available
    let has_context = app_state.codebase_contexts.lock().await
        .contains_key(meeting_id);

    if !has_context {
        info!("Codebase context not available yet, skipping auto task generation");
        return Ok(());
    }

    // Auto-trigger task generation
    info!("Auto-generating tasks from PRD");
    let tasks = generate_tasks_from_prd(
        meeting_id.to_string(),
        prd_version,
        State(&app_state),
    ).await?;

    info!("Generated {} tasks", tasks.len());
    Ok(())
}
```

## Frontend UI

```typescript
export const TaskGenerationView: React.FC<{ meetingId: string }> = ({ meetingId }) => {
  const [generating, setGenerating] = useState(false);
  const [tasks, setTasks] = useState<CodingTask[]>([]);

  const handleGenerateTasks = async () => {
    setGenerating(true);
    try {
      const generatedTasks = await invoke<CodingTask[]>("generate_tasks_from_prd", {
        meetingId,
        prdVersion: 1, // or get latest version
      });
      setTasks(generatedTasks);
      toast.success(`Generated ${generatedTasks.length} tasks`);
    } catch (error) {
      toast.error(`Failed to generate tasks: ${error}`);
    } finally {
      setGenerating(false);
    }
  };

  useEffect(() => {
    // Listen for auto-generated tasks
    const unlisten = listen<CodingTask[]>("tasks_generated", (event) => {
      setTasks(event.payload);
      toast.success(`${event.payload.length} tasks auto-generated`);
    });

    return () => { unlisten.then(fn => fn()); };
  }, []);

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2>Task Generation</h2>
        <Button onClick={handleGenerateTasks} disabled={generating}>
          {generating ? "Generating..." : "Generate Tasks from PRD"}
        </Button>
      </div>

      {tasks.length > 0 && (
        <div className="space-y-2">
          <p className="text-sm text-gray-600">
            Generated {tasks.length} tasks with dependencies resolved
          </p>
          {tasks.map(task => (
            <Card key={task.id}>
              <div className="font-medium">{task.title}</div>
              <div className="text-sm text-gray-600">{task.description}</div>
              <div className="text-xs text-gray-400 mt-2">
                Files: {[...task.files_to_create, ...task.files_to_modify].join(", ")}
              </div>
              {task.dependencies.length > 0 && (
                <div className="text-xs text-orange-500 mt-1">
                  Depends on {task.dependencies.length} other task(s)
                </div>
              )}
            </Card>
          ))}
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
    async fn test_generate_tasks_from_user_story() {
        let mock_prd = PRDContent {
            user_stories: vec![UserStory {
                title: "Add audio settings panel".to_string(),
                description: "Users should be able to configure microphone".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let mock_context = CodebaseContext {
            tech_stack: TechStack {
                primary_language: "TypeScript".to_string(),
                frameworks: vec!["React".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let generator = TaskGenerator::new(mock_claude_client());
        let tasks = generator.generate_from_prd(&mock_prd, &mock_context)
            .await
            .unwrap();

        assert!(!tasks.is_empty());
        assert!(tasks[0].title.contains("audio") || tasks[0].title.contains("settings"));
        assert!(!tasks[0].files_to_create.is_empty());
    }

    #[test]
    fn test_dependency_inference() {
        let mut tasks = vec![
            CodingTask {
                id: "1".to_string(),
                title: "Create UserAuth component".to_string(),
                files_to_create: vec!["src/auth/UserAuth.tsx".to_string()],
                ..Default::default()
            },
            CodingTask {
                id: "2".to_string(),
                title: "Add tests for UserAuth".to_string(),
                files_to_modify: vec!["src/auth/UserAuth.test.tsx".to_string()],
                ..Default::default()
            },
        ];

        let generator = TaskGenerator::new(mock_claude_client());
        generator.infer_dependencies(&mut tasks, &mock_context());

        assert!(tasks[1].dependencies.contains(&"1".to_string()));
    }
}
```

## File Structure

```
src-tauri/src/
├── task_generation/
│   ├── mod.rs                    # Module exports
│   ├── task_generator.rs         # Main generator (300 lines)
│   ├── llm_client.rs             # Claude API wrapper (150 lines)
│   ├── prompt_builder.rs         # Prompt construction (200 lines)
│   └── dependency_inferrer.rs    # Dependency logic (150 lines)
└── commands/
    └── task_generation.rs        # Tauri commands (100 lines)

src/components/meeting/
└── TaskGenerationView.tsx        # UI component (150 lines)
```

## Implementation Timeline

**Days 1-2:** LLM client and prompt templates
**Days 3-4:** Task generator core logic
**Days 5-6:** Dependency inference
**Days 7-8:** Auto-trigger integration with PRD
**Days 9-10:** Frontend UI and testing

## Error Handling

```rust
pub enum GenerationError {
    LLMError(String),
    ParseError(String),
    InvalidPRD(String),
    NoCodebaseContext,
}

// Retry logic for LLM failures
async fn generate_with_retry(&self, prompt: &str, max_retries: usize) -> Result<String> {
    for attempt in 1..=max_retries {
        match self.llm_client.chat(prompt).await {
            Ok(response) => return Ok(response),
            Err(e) if attempt < max_retries => {
                warn!("LLM request failed (attempt {}): {}", attempt, e);
                tokio::time::sleep(Duration::from_secs(2 * attempt as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

## Next Phase

Phase 4 implements the orchestrator that executes these generated tasks autonomously via Claude Code, with verification and auto-retry logic.
