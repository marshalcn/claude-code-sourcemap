// Placeholder for Anthropic API interaction
pub struct Client {
    pub api_key: String,
    // reqwest client
}

impl Client {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}
