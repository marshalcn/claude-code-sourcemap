use anyhow::Result;
use crate::tools::Tool;
use serde_json::json;

pub struct Agent {
    tools: Vec<Box<dyn Tool>>,
    // We would store history, context, and current config here
}

impl Agent {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    pub async fn run_single(&mut self, prompt: &str) -> Result<()> {
        println!("Agent received prompt: {}", prompt);
        
        // This is a temporary dummy execution to prove the tools work
        // In reality, the LLM will decide which tool to call and what arguments to pass
        
        if prompt.starts_with("/bash ") {
            let cmd = prompt.trim_start_matches("/bash ");
            println!(">> Executing Bash Tool...");
            if let Some(tool) = self.tools.iter().find(|t| t.name() == "bash") {
                let result = tool.execute(json!({ "command": cmd })).await;
                match result {
                    Ok(out) => println!("Result:\n{}", out),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        } else {
            println!("(To test bash tool manually, type: /bash <command>)");
        }
        
        Ok(())
    }
}
