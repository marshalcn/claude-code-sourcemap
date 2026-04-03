use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

pub struct SleepTool;

impl SleepTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct SleepArgs {
    duration_ms: u64,
}

#[async_trait]
impl Tool for SleepTool {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "Sleep".into()
    }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Pause agent execution for a specified duration. Use this to wait for background tasks, server startups, or external state changes without blocking system resources or the shell.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "duration_ms": {
                    "type": "integer",
                    "description": "The number of milliseconds to sleep (e.g. 5000 for 5 seconds)"
                }
            },
            "required": ["duration_ms"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: SleepArgs = serde_json::from_value(args)
            .context("Failed to parse SleepArgs")?;
            
        // Validate duration to prevent excessively long or infinite sleeps
        let duration = if args.duration_ms > 60000 {
            60000 // Cap at 1 minute for safety
        } else {
            args.duration_ms
        };
        
        sleep(Duration::from_millis(duration)).await;
        
        Ok(format!("Finished sleeping for {} milliseconds.", duration))
    }
}
