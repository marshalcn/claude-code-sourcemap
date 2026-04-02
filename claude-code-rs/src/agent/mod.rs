use anyhow::{Context, Result};
use crate::api::{Client, Message, Role, ContentBlock, CreateMessageRequest};
use crate::tools::Tool;
use std::collections::HashMap;

pub struct Agent {
    tools: HashMap<String, Box<dyn Tool>>,
    api_client: Option<Client>,
    conversation_history: Vec<Message>,
}

impl Agent {
    pub fn new() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let api_client = api_key.and_then(|k| Client::new(k).ok());

        Self {
            tools: HashMap::new(),
            api_client,
            conversation_history: Vec::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub async fn run_single(&mut self, prompt: &str) -> Result<()> {
        if self.api_client.is_none() {
            eprintln!("Warning: ANTHROPIC_API_KEY is not set. API calls will fail.");
            eprintln!("(Hint: export ANTHROPIC_API_KEY='sk-ant-...')\n");
        }

        println!("User: {}", prompt);
        
        self.conversation_history.push(Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: prompt.to_string() }],
        });

        // Run the agent loop (call API, execute tools, respond back)
        self.run_loop().await?;

        Ok(())
    }

    async fn run_loop(&mut self) -> Result<()> {
        let client = match &self.api_client {
            Some(c) => c,
            None => return Err(anyhow::anyhow!("API client is not initialized. Please set ANTHROPIC_API_KEY.")),
        };

        loop {
            let tools_def = self.tools.values().map(|t| t.as_tool_definition()).collect();

            let req = CreateMessageRequest {
                model: "claude-3-5-sonnet-20241022".to_string(), // Typical model used by Claude Code
                max_tokens: 4096,
                messages: self.conversation_history.clone(),
                system: Some("You are Claude Code, an AI coding assistant. You have access to tools to interact with the system.".to_string()),
                tools: tools_def,
            };

            println!("> Thinking...");
            let response = client.create_message(req).await?;
            
            // Add assistant's response to history
            self.conversation_history.push(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });

            let mut tool_results = Vec::new();
            let mut expects_tool_result = false;

            for block in &response.content {
                match block {
                    ContentBlock::Text { text } => {
                        println!("Claude: {}", text);
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        expects_tool_result = true;
                        println!("> Tool Call: {} ({})", name, input);
                        
                        let result_content = if let Some(tool) = self.tools.get(name) {
                            match tool.execute(input.clone()).await {
                                Ok(out) => {
                                    println!("> Tool Result: [Success]");
                                    out
                                },
                                Err(e) => {
                                    println!("> Tool Error: {}", e);
                                    format!("Error: {}", e)
                                }
                            }
                        } else {
                            format!("Error: Tool '{}' not found", name)
                        };

                        tool_results.push(ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: vec![ContentBlock::Text { text: result_content }],
                            is_error: None,
                        });
                    }
                    _ => {}
                }
            }

            if expects_tool_result {
                // If we executed tools, we must send the results back to Claude
                self.conversation_history.push(Message {
                    role: Role::User,
                    content: tool_results,
                });
            } else {
                // Stop the loop if no tools were called
                break;
            }
        }

        Ok(())
    }
}
