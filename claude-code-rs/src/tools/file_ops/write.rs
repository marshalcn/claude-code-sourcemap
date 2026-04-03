use crate::tools::Tool;
use crate::utils::{GLOBAL_PERMISSION_MANAGER, ActionType};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;

pub struct FileWriteTool;

impl FileWriteTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct FileWriteArgs {
    file_path: PathBuf,
    file_text: String,
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> std::borrow::Cow<'static, str> { "FileWrite".into() }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Write text to a file, overwriting its current contents.".into() 
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute or relative path to the file to write"
                },
                "file_text": {
                    "type": "string",
                    "description": "The complete text content to write into the file"
                }
            },
            "required": ["file_path", "file_text"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: FileWriteArgs = serde_json::from_value(args)
            .context("Failed to parse FileWriteArgs")?;

        let action = ActionType::FileWrite(args.file_path.to_string_lossy().into_owned());
        if !GLOBAL_PERMISSION_MANAGER.check_approval(&action).await? {
            return Ok("User denied permission to write to this file.".to_string());
        }
            
        fs::write(&args.file_path, &args.file_text)
            .await
            .context(format!("Failed to write file: {}", args.file_path.display()))?;
            
        Ok(format!("Successfully wrote {} bytes to {}", args.file_text.len(), args.file_path.display()))
    }
}
