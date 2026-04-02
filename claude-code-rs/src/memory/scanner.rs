use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::models::{MemoryFrontmatter, MemoryMetadata};

pub struct MemoryScanner {
    base_dir: PathBuf,
}

impl MemoryScanner {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Scans the memory directory, extracting up to `max_files` recent memories.
    pub fn scan(&self, max_files: usize) -> Result<Vec<MemoryMetadata>> {
        if !self.base_dir.exists() {
            // It's normal for the directory to not exist initially.
            return Ok(Vec::new());
        }

        let mut memories = Vec::new();
        let frontmatter_re = Regex::new(r"(?s)^---\s*\n(.*?)\n---").unwrap();

        for entry in WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let metadata = fs::metadata(entry.path())?;
            if metadata.is_dir() {
                continue;
            }

            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Parse frontmatter
            if let Some(captures) = frontmatter_re.captures(&content) {
                let yaml_str = captures.get(1).unwrap().as_str();
                if let Ok(frontmatter) = serde_yaml::from_str::<MemoryFrontmatter>(yaml_str) {
                    let modified_at: DateTime<Utc> = metadata.modified()?.into();
                    
                    memories.push(MemoryMetadata {
                        path: entry.path().to_path_buf(),
                        frontmatter,
                        modified_at,
                    });
                }
            }
        }

        // Sort by most recently modified
        memories.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        
        // Truncate to max_files
        if memories.len() > max_files {
            memories.truncate(max_files);
        }

        Ok(memories)
    }

    /// Generates a formatted text manifest of recent memories to inject into the side query.
    pub fn generate_manifest(&self, max_files: usize) -> Result<String> {
        let memories = self.scan(max_files)?;
        if memories.is_empty() {
            return Ok("No memories found.".to_string());
        }

        let mut manifest = String::from("Available Context & Memory Files:\n");
        for memory in memories {
            manifest.push_str(&memory.to_manifest_line());
            manifest.push('\n');
        }

        Ok(manifest)
    }
}
