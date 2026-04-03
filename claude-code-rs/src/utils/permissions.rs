use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

/// Represents the current permission mode of the Agent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    /// Every sensitive action requires explicit user approval
    Standard,
    /// YOLO Mode: Actions are automatically approved without prompting
    Yolo,
}

/// Represents a specific type of action that might need permission
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    BashCommand(String),
    FileWrite(String),
    FileEdit(String),
    NetworkRequest(String),
}

pub struct PermissionManager {
    mode: Arc<Mutex<PermissionMode>>,
    // In a real TUI, this would hold a channel to prompt the user
    // prompt_tx: mpsc::Sender<PermissionRequest>,
}

impl PermissionManager {
    pub fn new(initial_mode: PermissionMode) -> Self {
        Self {
            mode: Arc::new(Mutex::new(initial_mode)),
        }
    }

    /// Change the permission mode
    pub async fn set_mode(&self, mode: PermissionMode) {
        let mut current = self.mode.lock().await;
        *current = mode;
    }

    /// Check if the action is approved. If in Standard mode, it simulates
    /// prompting the user. If in YOLO mode, it automatically approves.
    pub async fn check_approval(&self, action: &ActionType) -> Result<bool> {
        let mode = *self.mode.lock().await;
        
        match mode {
            PermissionMode::Yolo => {
                // Auto-approve in YOLO mode
                Ok(true)
            }
            PermissionMode::Standard => {
                // In the original typescript code, this would render a `<PermissionPrompt>`
                // in the React Ink TUI and wait for the user to press 'y', 'n', or 'Escape'.
                // For this port, we will simulate asking for permission.
                
                let action_desc = match action {
                    ActionType::BashCommand(cmd) => format!("execute shell command: `{}`", cmd),
                    ActionType::FileWrite(path) => format!("write to file: {}", path),
                    ActionType::FileEdit(path) => format!("edit file: {}", path),
                    ActionType::NetworkRequest(url) => format!("make network request to: {}", url),
                };

                // NOTE: In a fully integrated TUI, we would pause execution here and
                // wait for a channel message from the TUI input loop.
                // For demonstration, we log the permission request and auto-approve it,
                // but in a production environment this MUST block.
                println!("\n[PERMISSION REQUEST] The agent wants to {}.", action_desc);
                println!("[PERMISSION REQUEST] (Auto-approved for mock purposes in Standard mode)");
                
                Ok(true) // Simulated approval
            }
        }
    }
}

lazy_static::lazy_static! {
    /// Global permission manager instance
    pub static ref GLOBAL_PERMISSION_MANAGER: PermissionManager = PermissionManager::new(PermissionMode::Standard);
}
