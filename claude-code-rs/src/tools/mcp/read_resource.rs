use crate::tools::Tool;
use crate::mcp::registry::GLOBAL_MCP_REGISTRY;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct ReadMcpResourceTool;

impl ReadMcpResourceTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ReadMcpResourceArgs {
    server: String,
    uri: String,
}

#[async_trait]
impl Tool for ReadMcpResourceTool {
    fn name(&self) -> std::borrow::Cow<'static, str> { "ReadMcpResource".into() }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Read the content of a specific Model Context Protocol (MCP) resource. Use ListMcpResources first to find the correct server name and URI.".into() 
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "Name of the MCP server providing the resource"
                },
                "uri": {
                    "type": "string",
                    "description": "The exact URI of the resource to read"
                }
            },
            "required": ["server", "uri"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: ReadMcpResourceArgs = serde_json::from_value(args)
            .context("Failed to parse ReadMcpResourceArgs")?;
            
        let contents = GLOBAL_MCP_REGISTRY.read_resource(&args.server, &args.uri).await?;
        
        if contents.is_empty() {
            return Err(anyhow::anyhow!("Resource {} not found on server {}", args.uri, args.server));
        }
        
        let mut output = String::new();
        for content in contents {
            if let Some(text) = content.text {
                output.push_str(&format!("--- Resource {} ({}) ---\n{}\n", content.uri, content.mime_type, text));
            } else if let Some(_blob) = content.blob {
                // In the original TS code, binary data is written to disk to prevent filling the context window.
                // For this port, we mock it by returning a generic success message instead of writing to disk.
                output.push_str(&format!("--- Resource {} ({}) ---\n[Binary Content - Not displaying directly]\n", content.uri, content.mime_type));
            }
        }
        
        Ok(output)
    }
}
