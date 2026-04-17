use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub subject: String,
    pub description: String,
    #[serde(rename = "activeForm")]
    pub active_form: Option<String>,
    pub status: TaskStatus,
    pub owner: Option<String>,
    pub blocks: Vec<String>,
    #[serde(rename = "blockedBy")]
    pub blocked_by: Vec<String>,
}

pub struct TaskManager {
    #[allow(dead_code)]
    tasks_dir: PathBuf,
    // Using an in-memory lock/state map to simulate file-level locking for this port
    memory_state: Arc<Mutex<HashMap<String, Task>>>,
    high_watermark: Arc<Mutex<u64>>,
}

impl TaskManager {
    pub fn new() -> Self {
        let tasks_dir = std::env::current_dir()
            .unwrap_or_default()
            .join(".claude")
            .join("tasks");
            
        Self {
            tasks_dir,
            memory_state: Arc::new(Mutex::new(HashMap::new())),
            high_watermark: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn create_task(&self, task: Task) -> Result<String> {
        let mut state = self.memory_state.lock().await;
        let mut hw = self.high_watermark.lock().await;
        
        *hw += 1;
        let new_id = hw.to_string();
        
        let mut new_task = task.clone();
        new_task.id = new_id.clone();
        
        state.insert(new_id.clone(), new_task);
        
        Ok(new_id)
    }

    pub async fn get_task(&self, id: &str) -> Option<Task> {
        let state = self.memory_state.lock().await;
        state.get(id).cloned()
    }

    pub async fn update_task(&self, id: &str, updates: TaskUpdate) -> Result<()> {
        let mut state = self.memory_state.lock().await;
        
        if let Some(task) = state.get_mut(id) {
            if let Some(subject) = updates.subject {
                task.subject = subject;
            }
            if let Some(description) = updates.description {
                task.description = description;
            }
            if let Some(status) = updates.status {
                task.status = status;
            }
            if let Some(owner) = updates.owner {
                task.owner = Some(owner);
            }
            if let Some(blocked_by) = updates.blocked_by {
                task.blocked_by = blocked_by;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}", id))
        }
    }

    pub async fn list_tasks(&self) -> Vec<Task> {
        let state = self.memory_state.lock().await;
        let mut tasks: Vec<Task> = state.values().cloned().collect();
        // Sort by ID
        tasks.sort_by(|a, b| {
            let a_id: u64 = a.id.parse().unwrap_or(0);
            let b_id: u64 = b.id.parse().unwrap_or(0);
            a_id.cmp(&b_id)
        });
        tasks
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_TASK_MANAGER: TaskManager = TaskManager::new();
}

#[derive(Deserialize)]
pub struct TaskUpdate {
    pub subject: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub owner: Option<String>,
    #[serde(rename = "blockedBy")]
    pub blocked_by: Option<Vec<String>>,
}
