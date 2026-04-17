use crate::tools::Tool;
use crate::tools::tasks::manager::{GLOBAL_TASK_MANAGER, TaskStatus};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct TaskListTool;

impl TaskListTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for TaskListTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "TaskList".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "List all tasks in the workspace, showing their IDs, status, subjects, and any blockers. Used to get a high-level overview of progress.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: Value) -> Result<String> {
        let tasks = GLOBAL_TASK_MANAGER.list_tasks().await;
        
        if tasks.is_empty() {
            return Ok("No tasks found".to_string());
        }
        
        let mut lines = Vec::new();
        for task in tasks {
            let owner = if let Some(o) = &task.owner { format!(" ({})", o) } else { String::new() };
            
            // Format status string
            let status_str = match task.status {
                TaskStatus::Pending => "pending",
                TaskStatus::InProgress => "in_progress",
                TaskStatus::Completed => "completed",
            };
            
            let blocked = if !task.blocked_by.is_empty() {
                format!(" [blocked by {}]", task.blocked_by.iter().map(|id| format!("#{}", id)).collect::<Vec<_>>().join(", "))
            } else {
                String::new()
            };
            
            lines.push(format!("#{} [{}] {}{}{}", task.id, status_str, task.subject, owner, blocked));
        }
        
        Ok(lines.join("\n"))
    }
}
