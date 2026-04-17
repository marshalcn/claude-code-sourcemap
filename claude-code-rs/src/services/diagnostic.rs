use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::mcp::McpClient;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub range: Range,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticFile {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

/// Service to track and compare code diagnostics (lint errors, warnings)
/// before and after file edits via MCP IDE integration.
pub struct DiagnosticTrackingService {
    baseline: Arc<Mutex<HashMap<String, Vec<Diagnostic>>>>,
    #[allow(dead_code)]
    initialized: Arc<Mutex<bool>>,
    // In a real implementation, this would hold an Arc to the connected MCP Client
    // to issue the `getDiagnostics` RPC call.
}

impl DiagnosticTrackingService {
    pub fn new() -> Self {
        Self {
            baseline: Arc::new(Mutex::new(HashMap::new())),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Reset tracking state for a new query loop
    pub async fn reset(&self) {
        let mut baseline = self.baseline.lock().await;
        baseline.clear();
    }

    fn normalize_file_uri(uri: &str) -> String {
        let prefixes = ["file://", "_claude_fs_right:", "_claude_fs_left:"];
        let mut normalized = uri;
        
        for prefix in prefixes {
            if uri.starts_with(prefix) {
                normalized = &uri[prefix.len()..];
                break;
            }
        }
        
        // Simple normalization for the port
        normalized.to_string()
    }

    /// Capture baseline diagnostics for a specific file before editing.
    pub async fn before_file_edited(&self, file_path: &str, mcp_client: &dyn McpClient) -> Result<()> {
        let uri = format!("file://{}", file_path);
        let normalized = Self::normalize_file_uri(file_path);
        
        // Call the IDE MCP server to get diagnostics
        // We mock the args formatting here based on the original TS code
        let args = serde_json::json!({ "uri": uri });
        
        match mcp_client.call_tool("getDiagnostics", args).await {
            Ok(result_str) => {
                if let Ok(parsed) = serde_json::from_str::<Vec<DiagnosticFile>>(&result_str) {
                    if let Some(file) = parsed.first() {
                        let mut baseline = self.baseline.lock().await;
                        baseline.insert(normalized, file.diagnostics.clone());
                    }
                }
            }
            Err(_) => {
                // Fail silently if IDE doesn't support diagnostics or isn't connected
                let mut baseline = self.baseline.lock().await;
                baseline.insert(normalized, Vec::new());
            }
        }
        Ok(())
    }

    /// Get new diagnostics that aren't in the baseline (e.g. after an edit)
    pub async fn get_new_diagnostics(&self, mcp_client: &dyn McpClient) -> Result<Vec<DiagnosticFile>> {
        // Fetch all diagnostics
        let args = serde_json::json!({});
        let result_str = match mcp_client.call_tool("getDiagnostics", args).await {
            Ok(res) => res,
            Err(_) => return Ok(Vec::new()), // Return empty if fetching fails
        };

        let all_files = match serde_json::from_str::<Vec<DiagnosticFile>>(&result_str) {
            Ok(f) => f,
            Err(_) => return Ok(Vec::new()),
        };

        let baseline = self.baseline.lock().await;
        let mut new_diagnostic_files = Vec::new();

        for file in all_files {
            if !file.uri.starts_with("file://") {
                continue;
            }

            let normalized = Self::normalize_file_uri(&file.uri);
            if let Some(baseline_diags) = baseline.get(&normalized) {
                // Filter out diagnostics that were already present in the baseline
                let new_diags: Vec<Diagnostic> = file.diagnostics.into_iter().filter(|d| {
                    !baseline_diags.iter().any(|b| b == d)
                }).collect();

                if !new_diags.is_empty() {
                    new_diagnostic_files.push(DiagnosticFile {
                        uri: file.uri.clone(),
                        diagnostics: new_diags,
                    });
                }
            }
        }

        Ok(new_diagnostic_files)
    }

    /// Format diagnostics into a human-readable summary string for the LLM
    pub fn format_diagnostics_summary(files: &[DiagnosticFile]) -> String {
        let mut result = String::new();
        
        for file in files {
            let filename = file.uri.split('/').last().unwrap_or(&file.uri);
            result.push_str(&format!("{}:\n", filename));
            
            for d in &file.diagnostics {
                let severity_symbol = match d.severity {
                    DiagnosticSeverity::Error => "❌",
                    DiagnosticSeverity::Warning => "⚠️",
                    DiagnosticSeverity::Info => "ℹ️",
                    DiagnosticSeverity::Hint => "💡",
                };
                
                let code_str = if let Some(ref c) = d.code { format!(" [{}]", c) } else { String::new() };
                let source_str = if let Some(ref s) = d.source { format!(" ({})", s) } else { String::new() };
                
                result.push_str(&format!(
                    "  {} [Line {}:{}] {}{}{}\n",
                    severity_symbol,
                    d.range.start.line + 1,
                    d.range.start.character + 1,
                    d.message,
                    code_str,
                    source_str
                ));
            }
            result.push('\n');
        }
        
        let max_chars = 4000;
        if result.len() > max_chars {
            let mut truncated = result[..max_chars].to_string();
            truncated.push_str("...[truncated]");
            truncated
        } else {
            result.trim_end().to_string()
        }
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_DIAGNOSTIC_TRACKER: DiagnosticTrackingService = DiagnosticTrackingService::new();
}
