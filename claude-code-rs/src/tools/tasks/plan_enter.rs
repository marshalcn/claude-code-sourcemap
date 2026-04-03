use crate::tools::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct EnterPlanModeTool;

impl EnterPlanModeTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for EnterPlanModeTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "EnterPlanMode".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Switch into plan mode to architect a solution or write a detailed plan before making any code changes. Use this when the task requires significant design, exploration, or multi-step execution. In plan mode, you are restricted to read-only operations (no file modifications).".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: Value) -> Result<String> {
        // In a complete implementation, this tool would signal the Agent or the context
        // to flip a state bit, effectively blocking all mutation tools (like FileEdit or Bash write commands)
        // until ExitPlanModeTool is called.
        // For this port, we return a success string that conceptually tells the LLM it's now in plan mode.
        Ok("Successfully entered plan mode. You are now restricted to read-only exploration tools. Gather information, analyze the codebase, and formulate a step-by-step plan. Once you are ready, use the ExitPlanMode tool to submit your plan and begin execution.".to_string())
    }
}
