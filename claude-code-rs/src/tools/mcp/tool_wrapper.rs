use crate::tools::Tool;
use crate::mcp::registry::GLOBAL_MCP_REGISTRY;
use anyhow::Result;
use async_trait::async_trait;

use serde_json::Value;

/// Dynamic wrapper for an MCP tool
pub struct McpToolWrapper {
    server_name: String,
    tool_name: String,
    description: String,
    input_schema: Value,
}

impl McpToolWrapper {
    pub fn new(server_name: String, tool_name: String, description: String, input_schema: Value) -> Self {
        Self {
            server_name,
            tool_name,
            description,
            input_schema,
        }
    }
}

#[async_trait]
impl Tool for McpToolWrapper {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.tool_name.clone().into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> {
        format!("[MCP Tool from server: {}] {}", self.server_name, self.description).into()
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn execute(&self, args: Value) -> Result<String> {
        GLOBAL_MCP_REGISTRY.call_tool(&self.server_name, &self.tool_name, args).await
    }
}
