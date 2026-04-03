use anyhow::Result;

pub struct AdvisorCommand;

impl AdvisorCommand {
    /// Handles the `/advisor` CLI command.
    ///
    /// `arg` is the argument passed after `/advisor` (e.g. "opus", "off", "unset").
    /// `current_advisor` is the currently configured advisor model (if any).
    /// `base_model` is the current main loop model (e.g. "claude-3-5-sonnet-20241022").
    ///
    /// Returns the display message to show the user and the new advisor model string
    /// (or None if it should be unset).
    pub fn handle_advisor(
        arg: &str,
        current_advisor: Option<&str>,
        base_model: &str,
    ) -> Result<(String, Option<String>)> {
        let arg = arg.trim().to_lowercase();
        
        let supports_advisor = base_model.contains("sonnet"); // Simplified check
        
        if arg.is_empty() {
            if let Some(current) = current_advisor {
                if !supports_advisor {
                    return Ok((format!("Advisor: {} (inactive)\nThe current model ({}) does not support advisors.", current, base_model), Some(current.to_string())));
                }
                return Ok((format!("Advisor: {}\nUse \"/advisor unset\" to disable or \"/advisor <model>\" to change.", current), Some(current.to_string())));
            } else {
                return Ok(("Advisor: not set\nUse \"/advisor <model>\" to enable (e.g. \"/advisor opus\").".to_string(), None));
            }
        }
        
        if arg == "unset" || arg == "off" {
            if let Some(prev) = current_advisor {
                return Ok((format!("Advisor disabled (was {}).", prev), None));
            } else {
                return Ok(("Advisor already unset.".to_string(), None));
            }
        }
        
        // Basic validation for the port
        let is_valid = arg.contains("claude") || arg.contains("opus") || arg.contains("haiku") || arg.contains("sonnet");
        
        if !is_valid {
            return Ok((format!("Invalid or unknown advisor model: {}", arg), current_advisor.map(String::from)));
        }
        
        // Assume normalization logic maps shorthand like "opus" to "claude-3-opus-20240229"
        let normalized_model = if arg == "opus" {
            "claude-3-opus-20240229".to_string()
        } else if arg == "haiku" {
            "claude-3-5-haiku-20241022".to_string()
        } else {
            arg.clone()
        };
        
        if !supports_advisor {
            return Ok((format!("Advisor set to {}.\nNote: Your current model ({}) does not support advisors. Switch to a supported model to use the advisor.", normalized_model, base_model), Some(normalized_model)));
        }
        
        Ok((format!("Advisor set to {}.", normalized_model), Some(normalized_model)))
    }
}
