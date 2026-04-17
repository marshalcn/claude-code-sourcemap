use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub web_search_requests: u64,
    pub cost_usd: f64,
    pub context_window: u64,
    pub max_output_tokens: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostState {
    pub total_cost_usd: f64,
    pub total_api_duration_ms: u64,
    pub total_tool_duration_ms: u64,
    pub total_lines_added: u64,
    pub total_lines_removed: u64,
    pub model_usage: HashMap<String, ModelUsage>,
    pub session_id: String,
}

pub struct CostTracker {
    state: Arc<Mutex<CostState>>,
}

impl CostTracker {
    pub fn new(session_id: String) -> Self {
        Self {
            state: Arc::new(Mutex::new(CostState {
                session_id,
                ..Default::default()
            })),
        }
    }

    /// Simulate the API usage calculation
    pub async fn add_api_usage(&self, model: &str, input_tokens: u64, output_tokens: u64, duration_ms: u64) {
        let mut state = self.state.lock().await;
        
        // Mock cost calculation: $3/M input, $15/M output for Sonnet (simplified)
        let cost_input = (input_tokens as f64 / 1_000_000.0) * 3.0;
        let cost_output = (output_tokens as f64 / 1_000_000.0) * 15.0;
        let total_cost = cost_input + cost_output;

        state.total_cost_usd += total_cost;
        state.total_api_duration_ms += duration_ms;

        let usage = state.model_usage.entry(model.to_string()).or_default();
        usage.input_tokens += input_tokens;
        usage.output_tokens += output_tokens;
        usage.cost_usd += total_cost;
    }

    pub async fn add_tool_duration(&self, duration_ms: u64) {
        let mut state = self.state.lock().await;
        state.total_tool_duration_ms += duration_ms;
    }

    pub async fn add_code_changes(&self, lines_added: u64, lines_removed: u64) {
        let mut state = self.state.lock().await;
        state.total_lines_added += lines_added;
        state.total_lines_removed += lines_removed;
    }

    pub async fn format_summary(&self) -> String {
        let state = self.state.lock().await;
        
        let mut result = format!(
            "Total cost:            ${:.4}\n\
             Total duration (API):  {:.2}s\n\
             Total duration (Tool): {:.2}s\n\
             Total code changes:    {} lines added, {} lines removed\n\
             Usage by model:\n",
            state.total_cost_usd,
            state.total_api_duration_ms as f64 / 1000.0,
            state.total_tool_duration_ms as f64 / 1000.0,
            state.total_lines_added,
            state.total_lines_removed
        );

        if state.model_usage.is_empty() {
            result.push_str("  0 input, 0 output, 0 cache read, 0 cache write");
        } else {
            for (model, usage) in &state.model_usage {
                result.push_str(&format!(
                    "  {}: {} input, {} output (${:.4})\n",
                    model, usage.input_tokens, usage.output_tokens, usage.cost_usd
                ));
            }
        }

        result
    }
}
