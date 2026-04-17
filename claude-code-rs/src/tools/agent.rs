use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct AgentTool;

impl AgentTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct AgentArgs {
    query: String,
    #[serde(default)]
    #[serde(rename = "subagent_type")]
    #[allow(dead_code)]
    subagent_type: Option<String>,
}

#[async_trait]
impl Tool for AgentTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "Agent".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Launch a sub-agent to handle complex multi-step tasks autonomously. Use this when a task is large or requires a specialized sub-agent (like exploration, verification, or code review).".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The task for the sub-agent to perform. Include all necessary context and a detailed plan."
                },
                "subagent_type": {
                    "type": "string",
                    "enum": ["explore", "plan", "verification"],
                    "description": "The specialized type of sub-agent to launch. Leave empty for a general-purpose agent."
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: AgentArgs = serde_json::from_value(args)
            .context("Failed to parse AgentArgs")?;
            
        // In the original typescript code, `AgentTool` triggers a completely new recursive queryLoop
        // via `queryModelWithStreaming` that spawns an inner Agent (with a specific persona and constraints).
        // It maintains isolation for the subagent's token context and returns only the final summarized output
        // back to the main agent.
        
        // For this Rust port, launching an actual recursive async agent requires sharing the `Client`
        // and setting up isolated context queues. We simulate this for now.
        let sub_type = args.subagent_type.unwrap_or_else(|| "general".to_string());
        
        Ok(format!(
            "Sub-Agent ({}) completed its task successfully.\n\n\
            Task Query: {}\n\n\
            (Note: In a full Rust implementation, this would spawn an isolated async Agent loop, \
            execute tools independently, and return a synthesized summary instead of this mock message.)",
            sub_type, args.query
        ))
    }
}
