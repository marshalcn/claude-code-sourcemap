pub mod bash;
pub mod file_ops;
pub mod ask_user;
pub mod web_fetch;
pub mod web_search;
pub mod notebook;
pub mod mcp;
pub mod tasks;
pub mod lsp;
pub mod agent;
pub mod config;
pub mod brief;
pub mod sleep;
pub mod tool_search;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::borrow::Cow;
use crate::api::ToolDefinition;

#[async_trait]
pub trait Tool: Send + Sync {
    /// Name of the tool as exposed to Claude
    fn name(&self) -> Cow<'static, str>;
    
    /// Description of what the tool does
    fn description(&self) -> Cow<'static, str>;
    
    /// Input schema for the tool (JSON Schema format)
    fn input_schema(&self) -> Value;
    
    /// Convert to Anthropic ToolDefinition
    fn as_tool_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.input_schema(),
        }
    }
    
    /// Execute the tool with given arguments
    async fn execute(&self, args: Value) -> Result<String>;
}

pub use bash::BashTool;
pub use file_ops::{FileReadTool, FileWriteTool, FileEditTool, GlobTool, GrepTool};
pub use ask_user::AskUserQuestionTool;
pub use web_fetch::WebFetchTool;
pub use web_search::WebSearchTool;
pub use notebook::NotebookEditTool;
pub use mcp::{ListMcpResourcesTool, ReadMcpResourceTool, McpToolWrapper};
pub use tasks::{TodoWriteTool, TaskCreateTool, TaskListTool, TaskUpdateTool, EnterPlanModeTool, ExitPlanModeTool};
pub use lsp::LspTool;
pub use agent::AgentTool;
pub use config::ConfigTool;
pub use brief::BriefTool;
pub use sleep::SleepTool;
pub use tool_search::ToolSearchTool;
