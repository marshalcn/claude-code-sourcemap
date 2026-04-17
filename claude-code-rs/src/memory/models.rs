use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The types of memory supported by the system.
/// This prevents the model from storing raw code or architecture that can be
/// easily derived by exploring the codebase directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    /// User persona, goals, responsibilities, and technical background. Always private.
    User,
    /// Corrections on AI behavior or confirmation of successful approaches.
    Feedback,
    /// Project context that cannot be derived directly from code (deadlines, compliance, etc).
    Project,
    /// Pointers to external systems (e.g., Jira, Linear, Grafana links).
    Reference,
}

/// Represents the YAML frontmatter parsed from a memory file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
}

/// Represents a scanned memory file containing metadata and its path.
#[derive(Debug, Clone)]
pub struct MemoryMetadata {
    pub path: PathBuf,
    pub frontmatter: MemoryFrontmatter,
    pub modified_at: DateTime<Utc>,
}

impl MemoryMetadata {
    /// Formats the metadata as a single line for the manifest.
    pub fn to_manifest_line(&self) -> String {
        format!(
            "- [{}]({}) - Type: {:?} - {}",
            self.frontmatter.name,
            self.path.display(),
            self.frontmatter.memory_type,
            self.frontmatter.description
        )
    }

    /// Calculates the age of the memory in days to help prevent hallucinations on stale code.
    pub fn age_in_days(&self) -> i64 {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.modified_at);
        duration.num_days()
    }

    /// Returns a warning string if the memory is old.
    pub fn age_warning(&self) -> Option<String> {
        let days = self.age_in_days();
        if days > 1 {
            Some(format!(
                "<system-reminder>\nThis memory is {} days old. Memories are point-in-time observations, not live state... Verify against current code before asserting as fact.\n</system-reminder>",
                days
            ))
        } else {
            None
        }
    }
}
