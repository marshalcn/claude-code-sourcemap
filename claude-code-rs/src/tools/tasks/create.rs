use crate::tools::Tool;
use crate::tools::tasks::manager::{Task, TaskStatus, GLOBAL_TASK_MANAGER};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct TaskCreateTool;

impl TaskCreateTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct TaskCreateArgs {
    subject: String,
    description: String,
    #[serde(default)]
    #[serde(rename = "activeForm")]
    active_form: Option<String>,
}

#[async_trait]
impl Tool for TaskCreateTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "TaskCreate".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Create a new task in the workspace task list. Use this to break down complex goals into trackable pieces.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "subject": {
                    "type": "string",
                    "description": "A brief title for the task"
                },
                "description": {
                    "type": "string",
                    "description": "What needs to be done"
                },
                "activeForm": {
                    "type": "string",
                    "description": "Present continuous form shown in spinner when in_progress (e.g., 'Running tests')"
                }
            },
            "required": ["subject", "description"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: TaskCreateArgs = serde_json::from_value(args)
            .context("Failed to parse TaskCreateArgs")?;
            
        let task = Task {
            id: String::new(), // Will be assigned by manager
            subject: args.subject.clone(),
            description: args.description,
            active_form: args.active_form,
            status: TaskStatus::Pending,
            owner: None,
            blocks: Vec::new(),
            blocked_by: Vec::new(),
        };

        let task_id = GLOBAL_TASK_MANAGER.create_task(task).await?;
        Ok(format!("Task #{} created successfully: {}", task_id, args.subject))
    }
}
