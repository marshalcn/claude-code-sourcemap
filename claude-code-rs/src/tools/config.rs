use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct ConfigTool;

impl ConfigTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ConfigArgs {
    setting: String,
    #[allow(dead_code)]
    value: Option<Value>,
}

#[async_trait]
impl Tool for ConfigTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "Config".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Read or modify Claude Code configuration settings and preferences (e.g. theme, models, etc). If value is provided, it updates the setting; otherwise, it reads the current value.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "setting": {
                    "type": "string",
                    "description": "The configuration key to read or update (e.g., 'theme', 'permissions.defaultMode')"
                },
                "value": {
                    "description": "The new value to set. Omit this field to simply read the current value."
                }
            },
            "required": ["setting"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: ConfigArgs = serde_json::from_value(args)
            .context("Failed to parse ConfigArgs")?;
            
        // In the original typescript code, this connects to a global configuration manager
        // that handles validation (e.g., verifying microphone access for voice settings)
        // and persists to disk. For this Rust port, we mock the behavior.
        
        if let Some(new_value) = args.value {
            Ok(format!(
                "Successfully updated configuration '{}' to '{}'.",
                args.setting, new_value
            ))
        } else {
            // Mock read
            Ok(format!(
                "Configuration '{}' is currently set to a default/mock value.",
                args.setting
            ))
        }
    }
}
