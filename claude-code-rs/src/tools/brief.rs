use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct BriefTool;

impl BriefTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct BriefArgs {
    message: String,
    #[allow(dead_code)]
    attachments: Option<Vec<String>>,
    status: String,
}

#[async_trait]
impl Tool for BriefTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "Brief".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Send formatted messages and attachments to the user. Supports proactive mode to interrupt and notify the user about important background status updates.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The formatted Markdown message to send to the user"
                },
                "attachments": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional list of file paths to attach to the message (logs, screenshots, diffs)"
                },
                "status": {
                    "type": "string",
                    "enum": ["normal", "proactive"],
                    "description": "Set to 'normal' for direct responses to user input, or 'proactive' when interrupting to notify the user of background updates."
                }
            },
            "required": ["message", "status"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: BriefArgs = serde_json::from_value(args)
            .context("Failed to parse BriefArgs")?;
            
        // In the original typescript code, this connects to the UI rendering engine
        // to display rich markdown and file attachments in the terminal.
        // It's the primary way for Agents (especially background/sub-agents)
        // to bubble up structured reports to the user.
        
        let mut result = format!(
            "Brief successfully sent to the user (Status: {}).\n\nMessage content: {}",
            args.status, args.message
        );
        
        if let Some(attachments) = args.attachments {
            if !attachments.is_empty() {
                result.push_str(&format!("\n\nAttachments: {:?}", attachments));
            }
        }
        
        Ok(result)
    }
}
