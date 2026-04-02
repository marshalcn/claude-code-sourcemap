use anyhow::Result;
use crate::tools::Tool;

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
        println!("Agent thinking about: {}", prompt);
        // Here we would construct the Anthropic API message
        // Send to Claude
        // Wait for tool_calls or response
        // Execute tools...
        Ok(())
    }
}
