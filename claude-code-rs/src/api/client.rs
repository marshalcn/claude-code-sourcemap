use super::types::{CreateMessageRequest, CreateMessageResponse};
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct Client {
    http_client: reqwest::Client,
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static(ANTHROPIC_VERSION));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        // Emulate typical headers to respect the official CLI behavior
        headers.insert("anthropic-beta", HeaderValue::from_static("tools-2024-04-04"));

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            http_client,
            api_key,
        })
    }

    pub async fn create_message(&self, req: CreateMessageRequest) -> Result<CreateMessageResponse> {
        let response = self.http_client
            .post(ANTHROPIC_API_URL)
            .json(&req)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("API Error {}: {}", status, error_body));
        }

        let resp_data: CreateMessageResponse = response
            .json()
            .await
            .context("Failed to deserialize API response")?;

        Ok(resp_data)
    }
}
