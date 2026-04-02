use super::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;

// --- File Read Tool ---

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
    fn name(&self) -> &'static str { "file_read" }
    
    fn description(&self) -> &'static str { 
        "Read the contents of a file. Returns file content." 
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

// --- File Write Tool ---

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
    fn name(&self) -> &'static str { "file_write" }
    
    fn description(&self) -> &'static str { 
        "Write text to a file, overwriting its current contents." 
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
            
        fs::write(&args.file_path, &args.file_text)
            .await
            .context(format!("Failed to write file: {}", args.file_path.display()))?;
            
        Ok(format!("Successfully wrote {} bytes to {}", args.file_text.len(), args.file_path.display()))
    }
}

// --- File Edit Tool (Search & Replace) ---

pub struct FileEditTool;

impl FileEditTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct FileEditArgs {
    file_path: PathBuf,
    old_str: String,
    new_str: String,
}

#[async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &'static str { "file_edit" }
    
    fn description(&self) -> &'static str { 
        "Edit a file by finding a specific string block and replacing it. This is useful for editing existing code." 
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "old_str": {
                    "type": "string",
                    "description": "The exact contiguous block of text to replace. Must match the file contents perfectly including whitespace and indentation."
                },
                "new_str": {
                    "type": "string",
                    "description": "The new block of text that will replace old_str."
                }
            },
            "required": ["file_path", "old_str", "new_str"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: FileEditArgs = serde_json::from_value(args)
            .context("Failed to parse FileEditArgs")?;
            
        let content = fs::read_to_string(&args.file_path)
            .await
            .context(format!("Failed to read file: {}", args.file_path.display()))?;
            
        if !content.contains(&args.old_str) {
            return Err(anyhow::anyhow!("The target string 'old_str' was not found in the file. Make sure indentation matches exactly."));
        }
        
        let new_content = content.replace(&args.old_str, &args.new_str);
        
        fs::write(&args.file_path, &new_content)
            .await
            .context(format!("Failed to write edited file: {}", args.file_path.display()))?;
            
        Ok(format!("Successfully edited {}", args.file_path.display()))
    }
}
