use anyhow::Result;
use crate::api::{Client, Message, Role, ContentBlock, CreateMessageRequest};
use crate::tools::Tool;
use crate::memory::MemoryRetriever;
use crate::services::{CostTracker, RateLimitTracker};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Message(String),
    Thinking,
    ToolCall(String, String),
    ToolResult(String, Result<String, String>),
    Error(String),
    Finished,
}

pub struct Agent {
    pub tools: HashMap<String, Box<dyn Tool>>,
    api_client: Option<Client>,
    pub conversation_history: Vec<Message>,
    memory_retriever: Option<MemoryRetriever>,
    pub cost_tracker: Arc<CostTracker>,
    pub rate_limit_tracker: Arc<RateLimitTracker>,
}

impl Agent {
    pub fn new() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let api_client = api_key.and_then(|k| Client::new(k).ok());
        
        let memory_retriever = if let Some(ref client) = api_client {
            // Check if .claude/memory directory exists, if not, it will be handled gracefully by scanner
            let memory_dir = std::env::current_dir().unwrap_or_default().join(".claude").join("memory");
            Some(MemoryRetriever::new(memory_dir, client.clone()))
        } else {
            None
        };

        Self {
            tools: HashMap::new(),
            api_client,
            conversation_history: Vec::new(),
            memory_retriever,
            cost_tracker: Arc::new(CostTracker::new("session_default".to_string())),
            rate_limit_tracker: Arc::new(RateLimitTracker::new()),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub async fn run_single(&mut self, prompt: &str, tx: Option<mpsc::Sender<AgentEvent>>) -> Result<()> {
        if self.api_client.is_none() {
            let msg = "Warning: ANTHROPIC_API_KEY is not set. API calls will fail.\n(Hint: export ANTHROPIC_API_KEY='sk-ant-...')\n";
            eprintln!("{}", msg);
            if let Some(ref tx) = tx {
                let _ = tx.send(AgentEvent::Error(msg.to_string())).await;
            }
        }

        if tx.is_none() {
            println!("User: {}", prompt);
        }
        
        let mut final_prompt = prompt.to_string();

        // Dynamically retrieve and inject relevant memories
        if let Some(ref retriever) = self.memory_retriever {
            if let Some(ref tx) = tx {
                let _ = tx.send(AgentEvent::Message("Scanning memories...".to_string())).await;
            }
            
            match retriever.find_relevant_memories(prompt, 5).await {
                Ok(memories) if !memories.is_empty() => {
                    let mut injected_context = String::from("\n\n<system-reminder>\nRelevant Context & Memories from your previous interactions:\n");
                    for mem in memories {
                        injected_context.push_str(&mem);
                    }
                    injected_context.push_str("\n</system-reminder>");
                    
                    final_prompt.push_str(&injected_context);
                    
                    if let Some(ref tx) = tx {
                        let _ = tx.send(AgentEvent::Message("Found relevant context from memory.".to_string())).await;
                    }
                }
                Ok(_) => {
                    // No relevant memories found
                }
                Err(e) => {
                    let err_msg = format!("Warning: Failed to retrieve memories: {}", e);
                    if let Some(ref tx) = tx {
                        let _ = tx.send(AgentEvent::Error(err_msg)).await;
                    } else {
                        eprintln!("{}", err_msg);
                    }
                }
            }
        }
        
        self.conversation_history.push(Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: final_prompt }],
        });

        // Run the agent loop (call API, execute tools, respond back)
        self.run_loop(tx.clone()).await?;

        if let Some(ref tx) = tx {
            let _ = tx.send(AgentEvent::Finished).await;
        }

        Ok(())
    }

    async fn run_loop(&mut self, tx: Option<mpsc::Sender<AgentEvent>>) -> Result<()> {
        let client = match &self.api_client {
            Some(c) => c,
            None => {
                let err = "API client is not initialized. Please set ANTHROPIC_API_KEY.";
                if let Some(ref tx) = tx {
                    let _ = tx.send(AgentEvent::Error(err.to_string())).await;
                }
                return Err(anyhow::anyhow!("{}", err));
            }
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

            if let Some(ref tx) = tx {
                let _ = tx.send(AgentEvent::Thinking).await;
            } else {
                println!("> Thinking...");
            }
            
            let start_time = Instant::now();
            let (response, headers) = client.create_message(req).await?;
            let duration_ms = start_time.elapsed().as_millis() as u64;
            
            // 1. Track API Limits
            let _ = self.rate_limit_tracker.extract_quota_status_from_headers(&headers).await;
            
            // 2. Track Token Cost
            // The Anthropic API response usually contains a `usage` block.
            // For this port, we will mock the token extraction.
            let input_tokens = 500; // Mock value, in real app extract from `response.usage.input_tokens`
            let output_tokens = 150; // Mock value, extract from `response.usage.output_tokens`
            self.cost_tracker.add_api_usage("claude-3-5-sonnet-20241022", input_tokens, output_tokens, duration_ms).await;
            
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
                        if let Some(ref tx) = tx {
                            let _ = tx.send(AgentEvent::Message(text.clone())).await;
                        } else {
                            println!("Claude: {}", text);
                        }
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        expects_tool_result = true;
                        
                        if let Some(ref tx) = tx {
                            let _ = tx.send(AgentEvent::ToolCall(name.clone(), input.to_string())).await;
                        } else {
                            println!("> Tool Call: {} ({})", name, input);
                        }
                        
                        let tool_start = Instant::now();
                        let (result_content, is_error) = if let Some(tool) = self.tools.get(name) {
                            // 3. Pre-tool hooks (e.g. Diagnostic Tracking before file edit)
                            // In a full implementation, we'd extract the file_path from `input`
                            // and call `GLOBAL_DIAGNOSTIC_TRACKER.before_file_edited(&file_path, mcp_client).await;`
                            
                            match tool.execute(input.clone()).await {
                                Ok(out) => {
                                    if let Some(ref tx) = tx {
                                        let _ = tx.send(AgentEvent::ToolResult(name.clone(), Ok(out.clone()))).await;
                                    } else {
                                        println!("> Tool Result: [Success]");
                                    }
                                    (out, false)
                                },
                                Err(e) => {
                                    let err_msg = format!("{}", e);
                                    if let Some(ref tx) = tx {
                                        let _ = tx.send(AgentEvent::ToolResult(name.clone(), Err(err_msg.clone()))).await;
                                    } else {
                                        println!("> Tool Error: {}", err_msg);
                                    }
                                    (format!("Error: {}", err_msg), true)
                                }
                            }
                        } else {
                            let err_msg = format!("Tool '{}' not found", name);
                            if let Some(ref tx) = tx {
                                let _ = tx.send(AgentEvent::ToolResult(name.clone(), Err(err_msg.clone()))).await;
                            } else {
                                println!("> Tool Error: {}", err_msg);
                            }
                            (format!("Error: {}", err_msg), true)
                        };
                        
                        let tool_duration = tool_start.elapsed().as_millis() as u64;
                        self.cost_tracker.add_tool_duration(tool_duration).await;
                        
                        // 4. Post-tool hooks (e.g. tracking changed lines, reading new diagnostics)
                        // self.cost_tracker.add_code_changes(added, removed).await;

                        tool_results.push(ContentBlock::ToolResult {
                            tool_use_id: id.clone(),
                            content: vec![ContentBlock::Text { text: result_content }],
                            is_error: if is_error { Some(true) } else { None },
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
