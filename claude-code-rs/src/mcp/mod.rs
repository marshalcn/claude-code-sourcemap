use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceInfo {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceContent {
    pub uri: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>, // Base64 encoded if binary
}

#[async_trait::async_trait]
pub trait McpClient: Send + Sync {
    /// Connect to the MCP server
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the MCP server
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Get list of tools provided by this server
    async fn list_tools(&self) -> Result<Vec<McpToolInfo>>;
    
    /// Call a specific tool on this server
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<String>;
    
    /// Get list of resources provided by this server
    async fn list_resources(&self) -> Result<Vec<McpResourceInfo>>;
    
    /// Read a specific resource from this server
    async fn read_resource(&self, uri: &str) -> Result<Vec<McpResourceContent>>;
}

pub mod stdio;
pub mod registry;

pub use registry::McpRegistry;
