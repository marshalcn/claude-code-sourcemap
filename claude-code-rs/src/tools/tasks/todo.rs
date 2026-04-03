use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub content: String,
    #[serde(rename = "activeForm")]
    pub active_form: String,
    pub status: TodoStatus,
}

#[derive(Deserialize)]
struct TodoWriteArgs {
    todos: Vec<TodoItem>,
}

pub struct TodoWriteTool {
    // In a real application, this might be tied to a specific session or persisted to disk.
    // For this port, we will store the current list in memory inside the tool itself.
    current_todos: Arc<Mutex<Vec<TodoItem>>>,
}

impl TodoWriteTool {
    pub fn new() -> Self {
        Self {
            current_todos: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Tool for TodoWriteTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "TodoWrite".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Update the todo list for the current session. To be used proactively and often to track progress and pending tasks. Make sure that at least one task is in_progress at all times. Always provide both content (imperative) and activeForm (present continuous) for each task.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "description": "The updated todo list",
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "The imperative form describing what needs to be done (e.g., 'Run tests')"
                            },
                            "activeForm": {
                                "type": "string",
                                "description": "The present continuous form shown during execution (e.g., 'Running tests')"
                            },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed"],
                                "description": "The current status of the task"
                            }
                        },
                        "required": ["content", "activeForm", "status"]
                    }
                }
            },
            "required": ["todos"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: TodoWriteArgs = serde_json::from_value(args)
            .context("Failed to parse TodoWriteArgs")?;
            
        let mut todos_guard = self.current_todos.lock().await;
        
        // Count how many tasks are completed
        let all_done = args.todos.iter().all(|t| t.status == TodoStatus::Completed);
        
        // Nudge logic: if closing out a 3+ item list and none of those items was a verification step, append a reminder
        let mut verification_nudge_needed = false;
        if all_done && args.todos.len() >= 3 {
            let has_verification = args.todos.iter().any(|t| t.content.to_lowercase().contains("verif"));
            if !has_verification {
                verification_nudge_needed = true;
            }
        }
        
        *todos_guard = if all_done { Vec::new() } else { args.todos };
        
        let mut result = String::from("Todos have been modified successfully. Ensure that you continue to use the todo list to track your progress. Please proceed with the current tasks if applicable.");
        
        if verification_nudge_needed {
            result.push_str("\n\nNOTE: You just closed out 3+ tasks and none of them was a verification step. Before writing your final summary, you should perform verification testing.");
        }
        
        Ok(result)
    }
}
