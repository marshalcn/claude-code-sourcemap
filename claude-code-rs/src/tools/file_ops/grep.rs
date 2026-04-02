use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct GrepArgs {
    pattern: String,
    path: Option<PathBuf>,
    include: Option<String>,
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str { "Grep" }
    
    fn description(&self) -> &'static str { 
        "- A powerful search tool
- Supports full regex syntax (e.g., \"log.*Error\", \"function\\s+\\w+\")
- Filter files with glob parameter (e.g., \"*.js\", \"**/*.tsx\")" 
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regular expression pattern to search for in file contents"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search in. Defaults to current working directory."
                },
                "include": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g. \"*.js\", \"*.{ts,tsx}\")"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: GrepArgs = serde_json::from_value(args)
            .context("Failed to parse GrepArgs")?;
            
        let search_path = args.path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        
        let absolute_search_path = if search_path.is_absolute() {
            search_path
        } else {
            std::env::current_dir()?.join(search_path)
        };

        if !absolute_search_path.exists() {
            return Err(anyhow::anyhow!("Directory or file does not exist: {}", absolute_search_path.display()));
        }

        // Using standard Command to call ripgrep (rg) or grep
        // In a production app, we would use the `grep` or `ignore` crate
        // or check if `rg` is installed and fallback to `grep`.
        
        let mut cmd = tokio::process::Command::new("rg");
        cmd.arg("-n"); // line numbers
        cmd.arg("-H"); // print filename
        cmd.arg("--no-heading"); // don't group by filename
        
        if let Some(ref include) = args.include {
            cmd.arg("-g").arg(&include);
        }
        
        cmd.arg(&args.pattern);
        cmd.arg(&absolute_search_path);

        let output = cmd.output().await;
        
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                if output.status.success() {
                    let lines: Vec<&str> = stdout.lines().collect();
                    let limit = 200;
                    
                    if lines.len() > limit {
                        let mut truncated = lines[0..limit].to_vec();
                        truncated.push("(Results are truncated. Consider using a more specific pattern or path.)");
                        Ok(truncated.join("\n"))
                    } else if lines.is_empty() {
                         Ok("No matches found".to_string())
                    } else {
                        Ok(stdout)
                    }
                } else if output.status.code() == Some(1) && stdout.is_empty() {
                    Ok("No matches found".to_string())
                } else {
                    // Fallback to standard grep if rg is not available
                    if stderr.contains("No such file or directory") {
                        let mut fallback_cmd = tokio::process::Command::new("grep");
                        fallback_cmd.arg("-rn");
                        
                        if let Some(include) = args.include {
                            fallback_cmd.arg("--include").arg(&include);
                        }
                        
                        fallback_cmd.arg(&args.pattern);
                        fallback_cmd.arg(&absolute_search_path);
                        
                        let fallback_output = fallback_cmd.output().await.context("Failed to execute grep fallback")?;
                        let fallback_stdout = String::from_utf8_lossy(&fallback_output.stdout).to_string();
                        
                        if fallback_output.status.success() {
                            let lines: Vec<&str> = fallback_stdout.lines().collect();
                            let limit = 200;
                            
                            if lines.len() > limit {
                                let mut truncated = lines[0..limit].to_vec();
                                truncated.push("(Results are truncated. Consider using a more specific pattern or path.)");
                                Ok(truncated.join("\n"))
                            } else if lines.is_empty() {
                                Ok("No matches found".to_string())
                            } else {
                                Ok(fallback_stdout)
                            }
                        } else {
                            Ok("No matches found".to_string())
                        }
                    } else {
                        Err(anyhow::anyhow!("Grep execution failed: {}", stderr))
                    }
                }
            }
            Err(e) => Err(anyhow::anyhow!("Failed to execute ripgrep/grep: {}", e)),
        }
    }
}