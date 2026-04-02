use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    /// Name of the tool as exposed to Claude
    fn name(&self) -> &'static str;
    
    /// Description of what the tool does
    fn description(&self) -> &'static str;
    
    /// Execute the tool with given arguments
    async fn execute(&self, args: Value) -> Result<String>;
}

pub struct BashTool;
impl BashTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str { "bash" }
    fn description(&self) -> &'static str { "Execute a bash command" }
    
    async fn execute(&self, _args: Value) -> Result<String> {
        // Implementation for executing bash commands safely
        Ok("Bash executed".to_string())
    }
}

pub struct FileEditTool;
impl FileEditTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &'static str { "file_edit" }
    fn description(&self) -> &'static str { "Edit a file" }
    
    async fn execute(&self, _args: Value) -> Result<String> {
        Ok("File edited".to_string())
    }
}
