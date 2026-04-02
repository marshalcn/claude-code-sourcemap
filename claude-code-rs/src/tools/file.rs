use super::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
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
        "Read the contents of a file. Returns file content. Requires { 'file_path': 'path/to/file' }" 
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
        "Write text to a file, overwriting its current contents. Requires { 'file_path': '...', 'file_text': '...' }" 
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
        "Edit a file by finding a specific string block and replacing it. Requires { 'file_path': '...', 'old_str': '...', 'new_str': '...' }" 
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: FileEditArgs = serde_json::from_value(args)
            .context("Failed to parse FileEditArgs")?;
            
        let content = fs::read_to_string(&args.file_path)
            .await
            .context(format!("Failed to read file: {}", args.file_path.display()))?;
            
        if !content.contains(&args.old_str) {
            return Err(anyhow::anyhow!("The target string 'old_str' was not found in the file."));
        }
        
        let new_content = content.replace(&args.old_str, &args.new_str);
        
        fs::write(&args.file_path, &new_content)
            .await
            .context(format!("Failed to write edited file: {}", args.file_path.display()))?;
            
        Ok(format!("Successfully edited {}", args.file_path.display()))
    }
}
