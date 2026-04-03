use crate::tools::Tool;
use crate::tools::tasks::manager::{GLOBAL_TASK_MANAGER, TaskStatus, TaskUpdate};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct TaskUpdateTool;

impl TaskUpdateTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct TaskUpdateArgs {
    id: String,
    subject: Option<String>,
    description: Option<String>,
    status: Option<TaskStatus>,
    owner: Option<String>,
    #[serde(rename = "blockedBy")]
    blocked_by: Option<Vec<String>>,
}

#[async_trait]
impl Tool for TaskUpdateTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "TaskUpdate".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Update an existing task in the task list. Change its status to in_progress or completed, reassign it, or update its subject/description/blockers.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "The unique ID of the task to update"
                },
                "subject": {
                    "type": "string",
                    "description": "A new brief title for the task"
                },
                "description": {
                    "type": "string",
                    "description": "A new detailed description of the task"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed"],
                    "description": "The new status of the task"
                },
                "owner": {
                    "type": "string",
                    "description": "The new owner assigned to the task"
                },
                "blockedBy": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Array of task IDs that block this task"
                }
            },
            "required": ["id"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: TaskUpdateArgs = serde_json::from_value(args)
            .context("Failed to parse TaskUpdateArgs")?;
            
        let update = TaskUpdate {
            subject: args.subject,
            description: args.description,
            status: args.status,
            owner: args.owner,
            blocked_by: args.blocked_by,
        };

        GLOBAL_TASK_MANAGER.update_task(&args.id, update).await?;
        
        Ok(format!("Task #{} updated successfully.", args.id))
    }
}
