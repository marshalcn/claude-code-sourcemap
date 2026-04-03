use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::fs;

pub struct ExitPlanModeTool;

impl ExitPlanModeTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ExitPlanModeArgs {
    plan: String,
}

#[async_trait]
impl Tool for ExitPlanModeTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "ExitPlanMode".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Submit a detailed plan and exit plan mode to start executing the changes. Your plan should clearly outline the steps, files to modify, and tests to run.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "plan": {
                    "type": "string",
                    "description": "A clear, actionable step-by-step plan based on your exploration and analysis"
                }
            },
            "required": ["plan"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: ExitPlanModeArgs = serde_json::from_value(args)
            .context("Failed to parse ExitPlanModeArgs")?;
            
        // In the original typescript code, the plan is persisted to `.claude/plan.md`.
        let plan_dir = std::env::current_dir()
            .unwrap_or_default()
            .join(".claude");
            
        if !plan_dir.exists() {
            fs::create_dir_all(&plan_dir).await.context("Failed to create .claude directory")?;
        }
        
        let plan_file_path = plan_dir.join("plan.md");
        fs::write(&plan_file_path, &args.plan).await.context("Failed to write plan to file")?;

        // This would signal the state machine to allow write-tools again.
        Ok(format!(
            "Successfully exited plan mode and saved the plan to {}. You are now back in standard execution mode and can use file modification tools to implement the steps.",
            plan_file_path.display()
        ))
    }
}
