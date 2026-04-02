use crate::api::{Client, CreateMessageRequest, Message, Role, ContentBlock};
use crate::memory::scanner::MemoryScanner;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

pub struct MemoryRetriever {
    scanner: MemoryScanner,
    api_client: Client,
}

impl MemoryRetriever {
    pub fn new(base_dir: PathBuf, api_client: Client) -> Self {
        Self {
            scanner: MemoryScanner::new(base_dir),
            api_client,
        }
    }

    /// Dynamically finds the most relevant memories for the current user query.
    /// Uses a side-query to the LLM (like Claude Sonnet) to pick the top N memories.
    pub async fn find_relevant_memories(
        &self,
        user_query: &str,
        max_files: usize,
    ) -> Result<Vec<String>> {
        let manifest = self.scanner.generate_manifest(200)?; // Read up to 200 recent memories
        
        if manifest.contains("No memories found") {
            return Ok(Vec::new());
        }

        // Create a prompt that asks the LLM to pick the top relevant files based on the manifest
        let system_prompt = "You are a context retrieval assistant. Your job is to read the provided manifest of memory files and select up to 5 file paths that are absolutely critical for answering the user's query. Return ONLY the file paths, separated by commas, with no other text.";
        let user_prompt = format!(
            "User Query:\n{}\n\nMemory Manifest:\n{}\n\nSelect up to {} relevant memory files.",
            user_query, manifest, max_files
        );

        let req = CreateMessageRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            system: Some(system_prompt.to_string()),
            messages: vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: user_prompt,
                }],
            }],
            tools: vec![], // No tools needed for this side-query
        };

        let response = self.api_client.create_message(req).await.context("Side-query failed")?;
        
        let mut relevant_contents = Vec::new();

        // Extract text from the LLM's response and parse the file paths
        for block in response.content {
            if let ContentBlock::Text { text } = block {
                let paths: Vec<&str> = text.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                
                for path_str in paths.into_iter().take(max_files) {
                    let path = PathBuf::from(path_str);
                    if path.exists() && path.is_file() {
                        let content = fs::read_to_string(&path).await.unwrap_or_default();
                        
                        // Check for age warning (prevent hallucination on stale code logic)
                        let metadata = std::fs::metadata(&path).ok();
                        let mut age_warning = String::new();
                        if let Some(m) = metadata {
                            if let Ok(sys_time) = m.modified() {
                                let datetime: chrono::DateTime<chrono::Utc> = sys_time.into();
                                let now = chrono::Utc::now();
                                let duration = now.signed_duration_since(datetime);
                                let days = duration.num_days();
                                if days > 1 {
                                    age_warning = format!("\n<system-reminder>\nThis memory is {} days old. Memories are point-in-time observations, not live state... Verify against current code before asserting as fact.\n</system-reminder>", days);
                                }
                            }
                        }

                        relevant_contents.push(format!(
                            "--- Memory: {} ---\n{}{}\n",
                            path.display(),
                            content,
                            age_warning
                        ));
                    }
                }
            }
        }

        Ok(relevant_contents)
    }
}
