use anyhow::Result;

pub struct BriefCommand;

impl BriefCommand {
    /// Simulates the logic of toggling brief mode.
    /// Returns the new state (true if brief mode is enabled, false if disabled)
    /// and the system reminder to inject into the LLM context.
    pub fn toggle_brief_mode(current_state: bool) -> Result<(bool, String, String)> {
        let new_state = !current_state;
        
        let system_reminder = if new_state {
            "<system-reminder>\nBrief mode is now enabled. Use the Brief tool for all user-facing output — plain text outside it is hidden from the user's view.\n</system-reminder>".to_string()
        } else {
            "<system-reminder>\nBrief mode is now disabled. The Brief tool is no longer available — reply with plain text.\n</system-reminder>".to_string()
        };
        
        let display_msg = if new_state {
            "Brief-only mode enabled".to_string()
        } else {
            "Brief-only mode disabled".to_string()
        };
        
        Ok((new_state, display_msg, system_reminder))
    }
}
