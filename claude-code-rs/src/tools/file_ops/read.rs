use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;

pub struct FileReadTool;

impl FileReadTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct FileReadArgs {
    file_path: PathBuf,
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> std::borrow::Cow<'static, str> { "FileRead".into() }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Read the contents of a file. Returns file content.".into() 
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute or relative path to the file to read"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: FileReadArgs = serde_json::from_value(args)
            .context("Failed to parse FileReadArgs")?;
            
        let content = fs::read_to_string(&args.file_path)
            .await
            .context(format!("Failed to read file: {}", args.file_path.display()))?;
            
        Ok(content)
    }
}
