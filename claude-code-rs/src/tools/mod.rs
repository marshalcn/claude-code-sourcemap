pub mod bash;
pub mod file;

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

pub use bash::BashTool;
pub use file::{FileReadTool, FileWriteTool, FileEditTool};
