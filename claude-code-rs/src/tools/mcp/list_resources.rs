use crate::tools::Tool;
use crate::mcp::registry::GLOBAL_MCP_REGISTRY;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct ListMcpResourcesTool;

impl ListMcpResourcesTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ListMcpResourcesArgs {
    #[serde(default)]
    server: Option<String>,
}

#[async_trait]
impl Tool for ListMcpResourcesTool {
    fn name(&self) -> std::borrow::Cow<'static, str> { "ListMcpResources".into() }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "List all available Model Context Protocol (MCP) resources. You can specify a server name to filter results, or leave it empty to list all resources across all connected MCP servers.".into() 
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "Optional name of the specific MCP server to list resources for. If omitted, lists resources from all connected servers."
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: ListMcpResourcesArgs = serde_json::from_value(args)
            .context("Failed to parse ListMcpResourcesArgs")?;
            
        let resources = GLOBAL_MCP_REGISTRY.list_all_resources().await?;
        
        if resources.is_empty() {
            return Ok("No MCP resources found. You may need to connect an MCP server first.".to_string());
        }
        
        let mut output = String::new();
        
        for (server_name, res) in resources {
            if let Some(ref target_server) = args.server {
                if target_server != &server_name {
                    continue;
                }
            }
            
            output.push_str(&format!("Server: {}\nURI: {}\nName: {}\n", server_name, res.uri, res.name));
            if let Some(desc) = res.description {
                output.push_str(&format!("Description: {}\n", desc));
            }
            if let Some(mime) = res.mime_type {
                output.push_str(&format!("MIME Type: {}\n", mime));
            }
            output.push_str("---\n");
        }
        
        if output.is_empty() {
            Ok(format!("No resources found for server: {}", args.server.unwrap_or_default()))
        } else {
            Ok(output)
        }
    }
}
