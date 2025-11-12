# Phase 2: Task Management System

## Overview

Build task queue infrastructure with dependency resolution. Provides the foundation for storing, prioritizing, and tracking coding tasks generated from PRDs.

## Prerequisites

- ✅ Phase 1: CodebaseContext available

## Goals

1. **Task Storage** - Persist tasks to disk with status tracking
2. **Dependency Management** - Resolve task order based on dependencies
3. **Priority Queue** - Execute high-priority tasks first
4. **Status Tracking** - pending → in_progress → completed/failed
5. **Basic UI** - View and manage task queue

## Success Criteria

- ✅ Handle 100+ tasks without performance degradation
- ✅ Detect and prevent circular dependencies
- ✅ Tasks persist across app restarts
- ✅ Dependency resolution is deterministic
- ✅ UI shows real-time task status updates

## Core Data Structures

### CodingTask

```rust
pub struct CodingTask {
    pub id: String,                    // UUID
    pub title: String,                 // "Create AudioSettingsPanel component"
    pub description: String,           // Detailed explanation
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,     // Task IDs that must complete first
    pub files_to_create: Vec<String>,
    pub files_to_modify: Vec<String>,
    pub prompt: String,                // Prompt for Claude Code
    pub implementation_context: Option<ImplementationContext>,
    pub attempts: Vec<TaskAttempt>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical,  // Infrastructure, breaking changes
    High,      // User-facing features
    Medium,    // Improvements, refactors
    Low,       // Nice-to-haves, polish
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,      // Not started
    Ready,        // Dependencies met, ready to execute
    InProgress,   // Currently executing
    Completed,    // Successfully done
    Failed,       // Failed after retries
    Blocked,      // Dependency failed
}
```

### TaskAttempt

```rust
pub struct TaskAttempt {
    pub attempt_number: usize,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub result: TaskResult,
    pub error_message: Option<String>,
    pub verification_output: Option<String>,
}

pub enum TaskResult {
    Success,
    CompilationError,
    TestFailure,
    Timeout,
    Unknown,
}
```

### ImplementationContext

```rust
pub struct ImplementationContext {
    pub similar_files: Vec<String>,     // Reference implementations
    pub integration_point: String,      // Where to integrate
    pub tech_stack_notes: String,       // Framework-specific guidance
    pub conventions: String,            // Naming/style patterns
}
```

## TaskQueue Implementation

### File: `src-tauri/src/task_management/task_queue.rs`

```rust
pub struct TaskQueue {
    tasks: HashMap<String, CodingTask>,
    storage_path: PathBuf,
}

impl TaskQueue {
    pub fn new(meeting_id: &str) -> Self {
        let storage_path = PathBuf::from(format!(
            "~/.handy/meetings/{}/tasks.json",
            meeting_id
        ));

        // Load existing tasks if present
        let tasks = if storage_path.exists() {
            Self::load_from_disk(&storage_path).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self { tasks, storage_path }
    }

    pub fn add_task(&mut self, task: CodingTask) -> Result<()> {
        self.tasks.insert(task.id.clone(), task);
        self.save_to_disk()?;
        Ok(())
    }

    pub fn get_next_ready_task(&self) -> Option<&CodingTask> {
        // Find highest priority task with status=Ready
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Ready)
            .max_by_key(|t| t.priority as u8)
    }

    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = status;
            task.updated_at = chrono::Utc::now().to_rfc3339();
            self.save_to_disk()?;
        }
        Ok(())
    }

    pub fn get_all_tasks(&self) -> Vec<&CodingTask> {
        self.tasks.values().collect()
    }

    fn save_to_disk(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.tasks)?;
        std::fs::write(&self.storage_path, json)?;
        Ok(())
    }
}
```

## Dependency Resolution

### File: `src-tauri/src/task_management/dependency_resolver.rs`

```rust
pub struct DependencyResolver {
    tasks: Vec<CodingTask>,
}

impl DependencyResolver {
    pub fn resolve_dependencies(&mut self) -> Result<(), DependencyError> {
        // 1. Build dependency graph
        let graph = self.build_graph()?;

        // 2. Detect cycles
        if let Some(cycle) = self.detect_cycles(&graph) {
            return Err(DependencyError::CircularDependency(cycle));
        }

        // 3. Update task statuses based on dependencies
        for task in &mut self.tasks {
            if self.are_dependencies_met(task) {
                task.status = TaskStatus::Ready;
            } else if self.any_dependency_failed(task) {
                task.status = TaskStatus::Blocked;
            } else {
                task.status = TaskStatus::Pending;
            }
        }

        Ok(())
    }

    fn are_dependencies_met(&self, task: &CodingTask) -> bool {
        task.dependencies.iter().all(|dep_id| {
            self.tasks
                .iter()
                .find(|t| &t.id == dep_id)
                .map(|t| t.status == TaskStatus::Completed)
                .unwrap_or(false)
        })
    }

    fn detect_cycles(&self, graph: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in graph.keys() {
            if self.has_cycle(node, graph, &mut visited, &mut rec_stack) {
                return Some(rec_stack.iter().cloned().collect());
            }
        }

        None
    }

    fn has_cycle(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }
}

pub enum DependencyError {
    CircularDependency(Vec<String>),
    MissingDependency(String),
}
```

## Tauri Commands

```rust
#[tauri::command]
async fn get_task_queue(
    meeting_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CodingTask>, String> {
    let queues = state.task_queues.lock().await;

    if let Some(queue) = queues.get(&meeting_id) {
        Ok(queue.get_all_tasks().into_iter().cloned().collect())
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
async fn update_task_status(
    meeting_id: String,
    task_id: String,
    status: TaskStatus,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut queues = state.task_queues.lock().await;

    if let Some(queue) = queues.get_mut(&meeting_id) {
        queue.update_task_status(&task_id, status)
            .map_err(|e| e.to_string())?;

        // Emit event for UI update
        state.app_handle.emit_all("task_status_updated", (&task_id, &status)).ok();
    }

    Ok(())
}

#[tauri::command]
async fn get_next_ready_task(
    meeting_id: String,
    state: State<'_, AppState>,
) -> Result<Option<CodingTask>, String> {
    let queues = state.task_queues.lock().await;

    if let Some(queue) = queues.get(&meeting_id) {
        Ok(queue.get_next_ready_task().cloned())
    } else {
        Ok(None)
    }
}
```

## Frontend UI

### Component: TaskQueueView

```typescript
interface Task {
  id: string;
  title: string;
  status: "pending" | "ready" | "in_progress" | "completed" | "failed" | "blocked";
  priority: "critical" | "high" | "medium" | "low";
  dependencies: string[];
}

export const TaskQueueView: React.FC<{ meetingId: string }> = ({ meetingId }) => {
  const [tasks, setTasks] = useState<Task[]>([]);

  useEffect(() => {
    // Load initial tasks
    invoke<Task[]>("get_task_queue", { meetingId }).then(setTasks);

    // Listen for updates
    const unlisten = listen("task_status_updated", () => {
      invoke<Task[]>("get_task_queue", { meetingId }).then(setTasks);
    });

    return () => { unlisten.then(fn => fn()); };
  }, [meetingId]);

  const tasksByStatus = {
    ready: tasks.filter(t => t.status === "ready"),
    in_progress: tasks.filter(t => t.status === "in_progress"),
    completed: tasks.filter(t => t.status === "completed"),
    pending: tasks.filter(t => t.status === "pending"),
    failed: tasks.filter(t => t.status === "failed"),
  };

  return (
    <div className="grid grid-cols-3 gap-4">
      <TaskColumn title="Ready" tasks={tasksByStatus.ready} color="blue" />
      <TaskColumn title="In Progress" tasks={tasksByStatus.in_progress} color="yellow" />
      <TaskColumn title="Completed" tasks={tasksByStatus.completed} color="green" />
    </div>
  );
};

const TaskColumn: React.FC<{ title: string; tasks: Task[]; color: string }> = ({
  title,
  tasks,
  color,
}) => (
  <div className="space-y-2">
    <h3 className="font-semibold">{title} ({tasks.length})</h3>
    {tasks.map(task => (
      <Card key={task.id} className={`border-l-4 border-${color}-500`}>
        <div className="text-sm font-medium">{task.title}</div>
        <div className="text-xs text-gray-500">Priority: {task.priority}</div>
        {task.dependencies.length > 0 && (
          <div className="text-xs text-gray-400">
            Depends on: {task.dependencies.length} task(s)
          </div>
        )}
      </Card>
    ))}
  </div>
);
```

## Testing Requirements

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_dependency_resolution() {
        let tasks = vec![
            CodingTask {
                id: "task1".to_string(),
                dependencies: vec!["task2".to_string()],
                status: TaskStatus::Pending,
                ..Default::default()
            },
            CodingTask {
                id: "task2".to_string(),
                dependencies: vec![],
                status: TaskStatus::Completed,
                ..Default::default()
            },
        ];

        let mut resolver = DependencyResolver { tasks };
        resolver.resolve_dependencies().unwrap();

        let task1 = resolver.tasks.iter().find(|t| t.id == "task1").unwrap();
        assert_eq!(task1.status, TaskStatus::Ready);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let tasks = vec![
            CodingTask {
                id: "task1".to_string(),
                dependencies: vec!["task2".to_string()],
                ..Default::default()
            },
            CodingTask {
                id: "task2".to_string(),
                dependencies: vec!["task1".to_string()],
                ..Default::default()
            },
        ];

        let mut resolver = DependencyResolver { tasks };
        let result = resolver.resolve_dependencies();

        assert!(matches!(result, Err(DependencyError::CircularDependency(_))));
    }

    #[test]
    fn test_task_queue_persistence() {
        let temp_dir = tempdir().unwrap();
        let queue_path = temp_dir.path().join("tasks.json");

        // Create queue and add task
        let mut queue = TaskQueue::new(queue_path.to_str().unwrap());
        queue.add_task(CodingTask {
            id: "test1".to_string(),
            title: "Test Task".to_string(),
            ..Default::default()
        }).unwrap();

        // Load in new instance
        let queue2 = TaskQueue::new(queue_path.to_str().unwrap());
        assert_eq!(queue2.get_all_tasks().len(), 1);
    }
}
```

## File Structure

```
src-tauri/src/
├── task_management/
│   ├── mod.rs                    # Module exports
│   ├── task_queue.rs             # Queue implementation (200 lines)
│   ├── dependency_resolver.rs   # Dependency logic (250 lines)
│   └── types.rs                  # Data structures (150 lines)
└── commands/
    └── tasks.rs                  # Tauri commands (150 lines)

src/components/meeting/
└── TaskQueueView.tsx             # UI component (250 lines)
```

## Implementation Timeline

**Days 1-2:** Data structures and basic TaskQueue
**Days 3-4:** Dependency resolver with cycle detection
**Days 5-6:** Tauri commands and storage
**Days 7-8:** Frontend UI with real-time updates
**Days 9-10:** Testing and edge cases

## Error Handling

```rust
pub enum TaskError {
    CircularDependency(Vec<String>),
    TaskNotFound(String),
    InvalidStatus(String),
    StorageError(std::io::Error),
}

// Log and notify user of errors
if let Err(e) = queue.resolve_dependencies() {
    error!("Dependency resolution failed: {:?}", e);
    app_handle.emit_all("task_error", e.to_string()).ok();
}
```

## Integration with Phase 3

Phase 3 (Task Generation) will populate the TaskQueue with tasks generated from PRD + CodebaseContext. Phase 2 provides:

- `TaskQueue::add_tasks(tasks: Vec<CodingTask>)` - Bulk add
- Automatic dependency resolution on add
- Status updates propagate to UI

## Next Phase

Phase 3 builds the LLM-powered task generator that converts PRD user stories into concrete CodingTasks with implementation context from Phase 1's codebase analysis.
