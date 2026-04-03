use crate::tools::Tool;
use crate::utils::{GLOBAL_PERMISSION_MANAGER, ActionType};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;

pub struct BashTool;

impl BashTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BashArgs {
    command: String,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> std::borrow::Cow<'static, str> { "Bash".into() }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Executes a bash command in the terminal. Returns the output (stdout and stderr) of the command.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: BashArgs = serde_json::from_value(args)
            .context("Failed to parse BashArgs. Expected { 'command': '...' }")?;

        // 1. Permission check
        let action = ActionType::BashCommand(args.command.clone());
        if !GLOBAL_PERMISSION_MANAGER.check_approval(&action).await? {
            return Ok("User denied permission to execute the bash command.".to_string());
        }

        // 2. Command execution
        let output = Command::new("bash")
            .arg("-c")
            .arg(&args.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context(format!("Failed to execute bash command: {}", args.command))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        let mut result = String::new();
        
        if !stdout.is_empty() {
            result.push_str("STDOUT:\n");
            result.push_str(&stdout);
            result.push('\n');
        }
        
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("STDERR:\n");
            result.push_str(&stderr);
            result.push('\n');
        }
        
        if result.is_empty() {
            if output.status.success() {
                result.push_str("Command executed successfully with no output.");
            } else {
                result.push_str(&format!("Command failed with exit code: {}", output.status));
            }
        }

        Ok(result)
    }
}
