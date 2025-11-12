# Phase 5: Intelligence & Production Safety

## Overview

Add advanced features for production reliability: LLM-powered code review, execution analytics, smart scheduling, and safety limits. This phase hardens the orchestration system for real-world use.

## Prerequisites

- ✅ Phase 1-4: Full orchestration pipeline working
- ✅ Tasks executing autonomously
- ✅ Verification and retry logic functioning

## Goals

1. **Code Quality** - LLM review before commit
2. **Analytics** - Track success rates and learn from failures
3. **Smart Scheduling** - Execute based on system load
4. **Safety Limits** - Circuit breakers and cost controls
5. **Performance** - Optimize for speed and resource usage

## Success Criteria

- ✅ Code review catches 90%+ of obvious issues
- ✅ 95%+ task success rate (up from 80% in Phase 4)
- ✅ Analytics provide actionable insights
- ✅ System stops safely when limits are hit
- ✅ Average task execution time reduced by 20%

## Feature 1: LLM Code Review

### Before Every Commit

```rust
pub struct CodeReviewer {
    llm_client: ClaudeClient,
}

impl CodeReviewer {
    pub async fn review_changes(
        &self,
        task: &CodingTask,
        changed_files: Vec<ChangedFile>,
    ) -> Result<ReviewResult, ReviewError> {
        let review_prompt = self.build_review_prompt(task, &changed_files);

        let response = self.llm_client.chat(
            &REVIEW_SYSTEM_PROMPT,
            &review_prompt,
            ClaudeModel::Haiku,  // Use Haiku for speed
        ).await?;

        let review: ReviewResult = serde_json::from_str(&response)?;
        Ok(review)
    }

    fn build_review_prompt(&self, task: &CodingTask, files: &[ChangedFile]) -> String {
        format!(
            r#"
TASK: {}

CHANGES:
{}

Review the code changes for:
1. Security vulnerabilities (SQL injection, XSS, etc.)
2. Obvious bugs (null checks, type errors, logic errors)
3. Performance issues (N+1 queries, memory leaks)
4. Code style consistency

Respond with JSON:
{{
  "approved": true/false,
  "issues": [
    {{"severity": "high|medium|low", "description": "...", "file": "...", "line": ...}}
  ],
  "suggestions": ["..."]
}}
"#,
            task.title,
            self.format_changes(files)
        )
    }
}

pub struct ReviewResult {
    pub approved: bool,
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<String>,
}

pub struct CodeIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub file: String,
    pub line: Option<usize>,
}

pub enum IssueSeverity {
    High,    // Security, major bugs
    Medium,  // Performance, minor bugs
    Low,     // Style, suggestions
}
```

### Integration with Orchestrator

```rust
impl CodeOrchestrator {
    async fn commit_changes(&self, task: &CodingTask) -> Result<(), OrchestrationError> {
        // Get changed files
        let changed_files = self.get_changed_files().await?;

        // Run code review
        let review = self.code_reviewer.review_changes(task, changed_files).await?;

        if !review.approved {
            // Review failed - block commit and retry task with review feedback
            warn!("Code review failed for task {}: {:?}", task.id, review.issues);

            // Add review feedback to task for retry
            task.prompt = self.add_review_feedback(task, &review);

            return Err(OrchestrationError::ReviewFailed(review));
        }

        // Review passed - proceed with commit
        self.git_commit(task).await?;
        Ok(())
    }
}
```

## Feature 2: Execution Analytics

### Track Everything

```rust
pub struct ExecutionAnalytics {
    storage_path: PathBuf,
}

pub struct AnalyticsData {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub success_rate: f64,
    pub average_attempts: f64,
    pub average_execution_time: Duration,
    pub common_failures: HashMap<String, usize>,
    pub task_type_success_rates: HashMap<String, f64>,
}

impl ExecutionAnalytics {
    pub fn record_task_completion(&mut self, task: &CodingTask) {
        let execution_time = self.calculate_execution_time(task);

        self.data.total_tasks += 1;

        if task.status == TaskStatus::Completed {
            self.data.completed_tasks += 1;
        } else {
            self.data.failed_tasks += 1;

            // Track failure reasons
            if let Some(last_attempt) = task.attempts.last() {
                if let Some(error) = &last_attempt.error_message {
                    let error_type = self.classify_error(error);
                    *self.data.common_failures.entry(error_type).or_insert(0) += 1;
                }
            }
        }

        self.data.average_attempts = self.calculate_average_attempts();
        self.data.success_rate = self.data.completed_tasks as f64 / self.data.total_tasks as f64;

        self.save().ok();
    }

    pub fn get_insights(&self) -> Vec<Insight> {
        let mut insights = Vec::new();

        // Low success rate
        if self.data.success_rate < 0.8 {
            insights.push(Insight {
                severity: InsightSeverity::High,
                message: format!(
                    "Success rate is low ({}%). Review common failure patterns.",
                    (self.data.success_rate * 100.0) as u32
                ),
                action: "Review task prompts and codebase context quality".to_string(),
            });
        }

        // High retry rate
        if self.data.average_attempts > 2.0 {
            insights.push(Insight {
                severity: InsightSeverity::Medium,
                message: "Tasks require multiple attempts. Consider improving prompts.".to_string(),
                action: "Add more context and examples to task prompts".to_string(),
            });
        }

        // Common failure patterns
        if let Some((error_type, count)) = self.data.common_failures.iter().max_by_key(|(_, c)| *c) {
            if *count > 3 {
                insights.push(Insight {
                    severity: InsightSeverity::Medium,
                    message: format!("Frequent {} errors ({}x)", error_type, count),
                    action: "Investigate root cause and improve verification".to_string(),
                });
            }
        }

        insights
    }
}
```

### Analytics Dashboard UI

```typescript
export const AnalyticsDashboard: React.FC<{ meetingId: string }> = ({ meetingId }) => {
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null);

  useEffect(() => {
    invoke<AnalyticsData>("get_analytics", { meetingId }).then(setAnalytics);
  }, [meetingId]);

  if (!analytics) return <div>Loading analytics...</div>;

  return (
    <div className="grid grid-cols-2 gap-4">
      <Card>
        <h3>Success Rate</h3>
        <div className="text-4xl font-bold">
          {(analytics.success_rate * 100).toFixed(1)}%
        </div>
        <Progress value={analytics.success_rate * 100} />
      </Card>

      <Card>
        <h3>Tasks Completed</h3>
        <div className="text-4xl font-bold">
          {analytics.completed_tasks} / {analytics.total_tasks}
        </div>
      </Card>

      <Card>
        <h3>Average Attempts</h3>
        <div className="text-4xl font-bold">
          {analytics.average_attempts.toFixed(1)}
        </div>
      </Card>

      <Card>
        <h3>Avg Execution Time</h3>
        <div className="text-4xl font-bold">
          {analytics.average_execution_time}s
        </div>
      </Card>

      <Card className="col-span-2">
        <h3>Common Failures</h3>
        {Object.entries(analytics.common_failures).map(([type, count]) => (
          <div key={type} className="flex justify-between">
            <span>{type}</span>
            <span className="font-bold">{count}x</span>
          </div>
        ))}
      </Card>
    </div>
  );
};
```

## Feature 3: Smart Scheduling

### Execute Based on System Load

```rust
pub struct SmartScheduler {
    max_concurrent_tasks: usize,
    max_cpu_percent: f32,
    max_memory_mb: usize,
}

impl SmartScheduler {
    pub async fn should_execute_next_task(&self) -> bool {
        // Check system resources
        let cpu_usage = self.get_cpu_usage().await;
        let memory_usage = self.get_memory_usage().await;
        let active_tasks = self.get_active_task_count().await;

        cpu_usage < self.max_cpu_percent
            && memory_usage < self.max_memory_mb
            && active_tasks < self.max_concurrent_tasks
    }

    pub async fn get_optimal_batch_size(&self) -> usize {
        let available_cpu = 100.0 - self.get_cpu_usage().await;
        let available_memory_percent = (self.max_memory_mb - self.get_memory_usage().await) as f32
            / self.max_memory_mb as f32;

        // Scale batch size based on available resources
        let resource_factor = (available_cpu / 100.0).min(available_memory_percent);

        (self.max_concurrent_tasks as f32 * resource_factor).max(1.0) as usize
    }

    async fn get_cpu_usage(&self) -> f32 {
        // Use sysinfo crate
        use sysinfo::{System, SystemExt};
        let mut sys = System::new_all();
        sys.refresh_cpu();
        sys.global_cpu_info().cpu_usage()
    }
}
```

### Integration

```rust
impl CodeOrchestrator {
    pub async fn start_with_smart_scheduling(&self) -> Result<()> {
        while self.is_running.load(Ordering::SeqCst) {
            // Check if we should execute next task
            if !self.scheduler.should_execute_next_task().await {
                info!("Waiting for resources to free up...");
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }

            // Execute batch of tasks
            let batch_size = self.scheduler.get_optimal_batch_size().await;
            let tasks = self.get_next_ready_tasks(batch_size).await;

            // Execute in parallel
            let futures: Vec<_> = tasks.into_iter()
                .map(|task| self.execute_task(task))
                .collect();

            futures::future::join_all(futures).await;
        }

        Ok(())
    }
}
```

## Feature 4: Safety Limits & Circuit Breakers

### Cost Controls

```rust
pub struct SafetyLimits {
    pub max_llm_requests_per_hour: usize,
    pub max_llm_tokens_per_day: usize,
    pub max_task_failures_before_stop: usize,
    pub max_execution_time_per_task: Duration,
}

pub struct CircuitBreaker {
    limits: SafetyLimits,
    llm_requests_this_hour: AtomicUsize,
    llm_tokens_today: AtomicUsize,
    consecutive_failures: AtomicUsize,
    last_reset: RwLock<Instant>,
}

impl CircuitBreaker {
    pub fn check_llm_request(&self) -> Result<(), CircuitBreakerError> {
        let current_requests = self.llm_requests_this_hour.load(Ordering::SeqCst);

        if current_requests >= self.limits.max_llm_requests_per_hour {
            return Err(CircuitBreakerError::RateLimitExceeded(
                "LLM requests per hour limit reached".to_string()
            ));
        }

        self.llm_requests_this_hour.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn record_failure(&self) -> Result<(), CircuitBreakerError> {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;

        if failures >= self.limits.max_task_failures_before_stop {
            return Err(CircuitBreakerError::TooManyFailures(failures));
        }

        Ok(())
    }

    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
    }

    pub async fn reset_hourly_counters(&self) {
        let mut last_reset = self.last_reset.write().await;
        let now = Instant::now();

        if now.duration_since(*last_reset) >= Duration::from_secs(3600) {
            self.llm_requests_this_hour.store(0, Ordering::SeqCst);
            *last_reset = now;
        }
    }
}
```

### Integration

```rust
impl CodeOrchestrator {
    async fn execute_task(&self, task: CodingTask) -> Result<()> {
        // Check circuit breaker
        self.circuit_breaker.check_llm_request()?;

        match self.execute_task_internal(task).await {
            Ok(_) => {
                self.circuit_breaker.record_success();
                Ok(())
            }
            Err(e) => {
                self.circuit_breaker.record_failure()?;
                Err(e)
            }
        }
    }
}
```

## Feature 5: Predictive Task Estimation

### Learn from History

```rust
pub struct TaskEstimator {
    historical_data: Vec<TaskExecution>,
}

pub struct TaskExecution {
    task_type: String,
    complexity_score: f64,
    execution_time: Duration,
    attempts_needed: usize,
}

impl TaskEstimator {
    pub fn estimate_task(&self, task: &CodingTask) -> TaskEstimate {
        let task_type = self.classify_task(task);
        let complexity = self.calculate_complexity(task);

        // Find similar historical tasks
        let similar_tasks: Vec<_> = self.historical_data.iter()
            .filter(|t| t.task_type == task_type)
            .filter(|t| (t.complexity_score - complexity).abs() < 0.3)
            .collect();

        if similar_tasks.is_empty() {
            return TaskEstimate::default();
        }

        let avg_time: Duration = similar_tasks.iter()
            .map(|t| t.execution_time)
            .sum::<Duration>() / similar_tasks.len() as u32;

        let avg_attempts: f64 = similar_tasks.iter()
            .map(|t| t.attempts_needed as f64)
            .sum::<f64>() / similar_tasks.len() as f64;

        TaskEstimate {
            estimated_time: avg_time,
            confidence: self.calculate_confidence(similar_tasks.len()),
            expected_attempts: avg_attempts.round() as usize,
        }
    }

    fn classify_task(&self, task: &CodingTask) -> String {
        if task.title.to_lowercase().contains("create") {
            "creation".to_string()
        } else if task.title.to_lowercase().contains("test") {
            "testing".to_string()
        } else if task.title.to_lowercase().contains("refactor") {
            "refactoring".to_string()
        } else {
            "modification".to_string()
        }
    }

    fn calculate_complexity(&self, task: &CodingTask) -> f64 {
        let mut score = 0.0;

        // More files = more complex
        score += (task.files_to_create.len() + task.files_to_modify.len()) as f64 * 0.3;

        // Dependencies = more complex
        score += task.dependencies.len() as f64 * 0.2;

        // Description length as proxy for complexity
        score += (task.description.len() / 100) as f64 * 0.1;

        score.min(10.0)  // Cap at 10
    }
}
```

## Tauri Commands

```rust
#[tauri::command]
async fn get_analytics(meeting_id: String, state: State<'_, AppState>) -> Result<AnalyticsData> {
    let analytics = state.analytics.lock().await;
    Ok(analytics.get_data(&meeting_id))
}

#[tauri::command]
async fn get_insights(meeting_id: String, state: State<'_, AppState>) -> Result<Vec<Insight>> {
    let analytics = state.analytics.lock().await;
    Ok(analytics.get_insights())
}

#[tauri::command]
async fn update_safety_limits(limits: SafetyLimits, state: State<'_, AppState>) -> Result<()> {
    let mut circuit_breaker = state.circuit_breaker.write().await;
    circuit_breaker.update_limits(limits);
    Ok(())
}
```

## File Structure

```
src-tauri/src/
├── intelligence/
│   ├── mod.rs                    # Module exports
│   ├── code_reviewer.rs          # LLM review (200 lines)
│   ├── analytics.rs              # Execution tracking (250 lines)
│   ├── smart_scheduler.rs        # Resource-aware scheduling (150 lines)
│   ├── circuit_breaker.rs        # Safety limits (200 lines)
│   └── task_estimator.rs         # Predictive estimation (150 lines)
└── commands/
    └── intelligence.rs           # Tauri commands (100 lines)

src/components/meeting/
└── AnalyticsDashboard.tsx        # UI component (300 lines)
```

## Implementation Timeline

**Days 1-2:** LLM code review integration
**Days 3-4:** Execution analytics and tracking
**Days 5-6:** Smart scheduler with resource monitoring
**Days 7-8:** Circuit breakers and safety limits
**Days 9-10:** Task estimation and final testing

## Testing Requirements

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_code_review_detects_issues() {
        let reviewer = CodeReviewer::new(mock_llm_client());
        let task = create_mock_task();
        let files = vec![ChangedFile {
            path: "test.ts".to_string(),
            content: "let x; console.log(x.toString())".to_string(),  // Null ref
        }];

        let review = reviewer.review_changes(&task, files).await.unwrap();

        assert!(!review.approved);
        assert!(!review.issues.is_empty());
    }

    #[test]
    fn test_circuit_breaker_triggers() {
        let breaker = CircuitBreaker::new(SafetyLimits {
            max_task_failures_before_stop: 3,
            ..Default::default()
        });

        breaker.record_failure().ok();
        breaker.record_failure().ok();

        let result = breaker.record_failure();
        assert!(matches!(result, Err(CircuitBreakerError::TooManyFailures(_))));
    }
}
```

## Performance Optimizations

1. **Parallel Execution**: Execute independent tasks concurrently
2. **Caching**: Cache LLM responses for similar prompts
3. **Batch Processing**: Group similar tasks for efficient execution
4. **Resource Pooling**: Reuse connections and clients

## Configuration UI

```typescript
export const SafetySettings: React.FC = () => {
  const [limits, setLimits] = useState<SafetyLimits>({
    max_llm_requests_per_hour: 100,
    max_llm_tokens_per_day: 100000,
    max_task_failures_before_stop: 5,
    max_execution_time_per_task: 300,
  });

  const handleSave = async () => {
    await invoke("update_safety_limits", { limits });
    toast.success("Safety limits updated");
  };

  return (
    <Card>
      <h3>Safety Limits</h3>
      <div className="space-y-4">
        <Input
          label="Max LLM Requests/Hour"
          type="number"
          value={limits.max_llm_requests_per_hour}
          onChange={(e) => setLimits({
            ...limits,
            max_llm_requests_per_hour: parseInt(e.target.value)
          })}
        />
        <Input
          label="Max Failures Before Stop"
          type="number"
          value={limits.max_task_failures_before_stop}
          onChange={(e) => setLimits({
            ...limits,
            max_task_failures_before_stop: parseInt(e.target.value)
          })}
        />
        <Button onClick={handleSave}>Save Limits</Button>
      </div>
    </Card>
  );
};
```

## Production Readiness Checklist

- ✅ Code review catches security issues
- ✅ Analytics track all executions
- ✅ Circuit breakers prevent runaway costs
- ✅ Smart scheduling prevents resource exhaustion
- ✅ Task estimation provides realistic timelines
- ✅ Comprehensive error logging
- ✅ User-configurable safety limits
- ✅ Graceful degradation on failures

## Final Integration

With Phase 5 complete, the full orchestration system is production-ready:
1. Meeting starts → Codebase analyzed in parallel
2. PRD generated from conversation
3. Tasks generated with context
4. Orchestrator executes with verification
5. Code reviewed before commit
6. Analytics track performance
7. Safety limits prevent issues
8. User receives working code at meeting end

**The vision is complete: Conversations become code automatically.**
