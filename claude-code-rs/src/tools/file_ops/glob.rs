use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct GlobArgs {
    pattern: String,
    path: Option<PathBuf>,
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &'static str { "Glob" }
    
    fn description(&self) -> &'static str { 
        "- Fast file pattern matching tool that works with any codebase size
- Supports glob patterns like \"/*.js\" or \"src/**/*.ts\"
- Returns matching file paths sorted by modification time
- Use this tool when you need to find files by name patterns" 
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The glob pattern to match files against."
                },
                "path": {
                    "type": "string",
                    "description": "The directory to search in. If not specified, the current working directory will be used. Must be a valid absolute directory path if provided."
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: GlobArgs = serde_json::from_value(args)
            .context("Failed to parse GlobArgs")?;
            
        let search_path = args.path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        
        let absolute_search_path = if search_path.is_absolute() {
            search_path
        } else {
            std::env::current_dir()?.join(search_path)
        };

        if !absolute_search_path.exists() {
            return Err(anyhow::anyhow!("Directory does not exist: {}", absolute_search_path.display()));
        }
        if !absolute_search_path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {}", absolute_search_path.display()));
        }

        let full_pattern = if args.pattern.starts_with('/') {
            format!("{}{}", absolute_search_path.display(), args.pattern)
        } else {
            format!("{}/{}", absolute_search_path.display(), args.pattern)
        };

        let mut results = Vec::new();
        
        let glob_results = match glob::glob(&full_pattern) {
            Ok(paths) => paths,
            Err(e) => return Err(anyhow::anyhow!("Invalid glob pattern: {}", e)),
        };

        for entry in glob_results {
            match entry {
                Ok(path) => {
                    if let Ok(current_dir) = std::env::current_dir() {
                        if let Ok(rel_path) = path.strip_prefix(&current_dir) {
                            results.push(rel_path.display().to_string());
                        } else {
                            results.push(path.display().to_string());
                        }
                    } else {
                        results.push(path.display().to_string());
                    }
                }
                Err(e) => eprintln!("Glob error: {:?}", e),
            }
        }

        if results.is_empty() {
            Ok("No files found".to_string())
        } else {
            let limit = 100;
            let truncated = results.len() > limit;
            
            if truncated {
                results.truncate(limit);
                results.push("(Results are truncated. Consider using a more specific path or pattern.)".to_string());
            }
            
            Ok(results.join("\n"))
        }
    }
}
