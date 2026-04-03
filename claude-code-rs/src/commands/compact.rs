use anyhow::Result;
use crate::api::Message;

pub struct CompactCommand;

impl CompactCommand {
    /// Handles the `/compact` CLI command.
    /// In a full implementation, this would run a background LLM summarization
    /// task to compress the `conversation_history` into a single summary message.
    /// For this port, we simulate the compaction by keeping only the system prompt
    /// and the most recent N messages, replacing the rest with a placeholder.
    pub async fn handle_compact(
        args: &str,
        history: &mut Vec<Message>,
    ) -> Result<String> {
        if history.len() <= 2 {
            return Ok("No messages to compact. Conversation is already short.".to_string());
        }

        // We keep the first message (usually system prompt or initial user query)
        // and the last 2 messages. The rest are summarized.
        let retain_count = 2;
        if history.len() > retain_count + 1 {
            let first = history[0].clone();
            let mut recent = history.split_off(history.len() - retain_count);
            
            history.clear();
            history.push(first);
            
            // Insert a mock summary message
            let summary_text = if args.is_empty() {
                "[System: Conversation compacted. Older messages have been removed to save tokens.]"
            } else {
                &format!("[System: Conversation compacted with instructions: '{}']", args)
            };
            
            history.push(Message {
                role: crate::api::Role::Assistant,
                content: vec![crate::api::ContentBlock::Text { text: summary_text.to_string() }],
            });
            
            history.append(&mut recent);
            
            Ok(format!("Compacted {} messages.", history.len() - retain_count))
        } else {
            Ok("Conversation too short to compact.".to_string())
        }
    }
}
