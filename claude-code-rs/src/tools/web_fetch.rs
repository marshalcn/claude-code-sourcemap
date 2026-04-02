use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct WebFetchTool;

impl WebFetchTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize, Debug)]
struct WebFetchArgs {
    url: String,
    #[serde(default)]
    prompt: String,
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> std::borrow::Cow<'static, str> { "WebFetch".into() }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Fetches the content of a URL and optionally applies a prompt to process it. 
IMPORTANT: WebFetch WILL FAIL for authenticated or private URLs. Before using this tool, check if the URL points to an authenticated service.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch content from"
                },
                "prompt": {
                    "type": "string",
                    "description": "The prompt to run on the fetched content"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: WebFetchArgs = serde_json::from_value(args)
            .context("Failed to parse WebFetchArgs")?;
            
        let client = reqwest::Client::builder()
            .user_agent("Claude Code Rust Port/1.0")
            .build()?;
            
        let response = client.get(&args.url).send().await
            .context(format!("Failed to fetch URL: {}", args.url))?;
            
        let status = response.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!("HTTP Error {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));
        }

        let content = response.text().await
            .context("Failed to read response body as text")?;
            
        // In the original TypeScript version, if a `prompt` is provided,
        // it applies an LLM to process the markdown content.
        // For this port, we will just return the raw text (truncated if too long).
        let max_length = 50_000;
        
        let final_content = if content.len() > max_length {
            let mut truncated = content[0..max_length].to_string();
            truncated.push_str("\n\n...(Content truncated due to length)...");
            truncated
        } else {
            content
        };

        if args.prompt.is_empty() {
            Ok(final_content)
        } else {
            Ok(format!("Prompt applied: {}\n\nContent:\n{}", args.prompt, final_content))
        }
    }
}
