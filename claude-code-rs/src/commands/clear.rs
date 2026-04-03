use anyhow::Result;
use crate::api::Message;

pub struct ClearCommand;

impl ClearCommand {
    /// Handles the `/clear` CLI command.
    /// This simply clears the `conversation_history` (except for any persistent
    /// system instructions if they were stored at index 0).
    pub fn handle_clear(history: &mut Vec<Message>) -> Result<String> {
        let cleared_count = history.len();
        
        if cleared_count == 0 {
            return Ok("Conversation is already empty.".to_string());
        }

        // In Claude Code, clearing usually retains the initial system instructions
        // if they are part of the messages array.
        // For this port, we clear everything since the system prompt is often
        // injected per request.
        history.clear();
        
        Ok(format!("Conversation cleared (removed {} messages).", cleared_count))
    }
}
