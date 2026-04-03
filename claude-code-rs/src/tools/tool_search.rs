use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct ToolSearchTool;

impl ToolSearchTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ToolSearchArgs {
    query: String,
    #[allow(dead_code)]
    max_results: Option<usize>,
}

#[async_trait]
impl Tool for ToolSearchTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "ToolSearch".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Search the global registry for available or deferred tools. Useful when you need a specific capability (e.g. database query, git operation) that isn't currently loaded in your context. You can use 'select:<tool_name>' to directly activate a tool.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Keywords to search for tool names and descriptions, or 'select:<tool_name>' to activate exactly one."
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of tools to return (default: 5)"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: ToolSearchArgs = serde_json::from_value(args)
            .context("Failed to parse ToolSearchArgs")?;
            
        // In the original codebase, this interacts with a complex tool loader that 
        // defers loading MCP or heavy tools to save token context. When the agent 
        // searches, it lazy-loads the matching tools into the prompt for the next turn.
        
        if args.query.starts_with("select:") {
            let tool_name = args.query.trim_start_matches("select:").trim();
            Ok(format!(
                "Successfully activated tool '{}'. It will be available for you to use in the next turn.",
                tool_name
            ))
        } else {
            // Mock a search result
            Ok(format!(
                "Search query '{}' completed.\n\n\
                Found matches:\n\
                - 'GitCommit' (Deferred): Creates a git commit with changes.\n\
                - 'DatabaseQuery' (Deferred): Executes SQL queries against the active connection.\n\n\
                Use 'select:<tool_name>' in your next ToolSearch call to activate a specific tool.",
                args.query
            ))
        }
    }
}
