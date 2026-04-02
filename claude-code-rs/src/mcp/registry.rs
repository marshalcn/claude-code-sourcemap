use super::{McpClient, McpToolInfo, McpResourceInfo, McpResourceContent};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;

pub struct McpRegistry {
    clients: Arc<RwLock<HashMap<String, Box<dyn McpClient>>>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_client(&self, name: String, mut client: Box<dyn McpClient>) -> Result<()> {
        // Automatically connect upon registration
        client.connect().await.context(format!("Failed to connect to MCP server: {}", name))?;
        
        let mut guard = self.clients.write().await;
        guard.insert(name, client);
        
        Ok(())
    }

    pub async fn list_all_tools(&self) -> Result<Vec<(String, McpToolInfo)>> {
        let mut all_tools = Vec::new();
        let guard = self.clients.read().await;
        
        for (server_name, client) in guard.iter() {
            if let Ok(tools) = client.list_tools().await {
                for tool in tools {
                    all_tools.push((server_name.clone(), tool));
                }
            }
        }
        
        Ok(all_tools)
    }

    pub async fn call_tool(&self, server_name: &str, tool_name: &str, arguments: Value) -> Result<String> {
        let guard = self.clients.read().await;
        
        if let Some(client) = guard.get(server_name) {
            client.call_tool(tool_name, arguments).await
        } else {
            Err(anyhow::anyhow!("MCP server '{}' not found", server_name))
        }
    }

    pub async fn list_all_resources(&self) -> Result<Vec<(String, McpResourceInfo)>> {
        let mut all_resources = Vec::new();
        let guard = self.clients.read().await;
        
        for (server_name, client) in guard.iter() {
            if let Ok(resources) = client.list_resources().await {
                for res in resources {
                    all_resources.push((server_name.clone(), res));
                }
            }
        }
        
        Ok(all_resources)
    }

    pub async fn read_resource(&self, server_name: &str, uri: &str) -> Result<Vec<McpResourceContent>> {
        let guard = self.clients.read().await;
        
        if let Some(client) = guard.get(server_name) {
            client.read_resource(uri).await
        } else {
            Err(anyhow::anyhow!("MCP server '{}' not found", server_name))
        }
    }
}

// Global registry instance (in a real app, you might inject this instead of using lazy_static/OnceCell)
lazy_static::lazy_static! {
    pub static ref GLOBAL_MCP_REGISTRY: McpRegistry = McpRegistry::new();
}
