use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContestInfo{
    pub submit_url: String,
    pub language_id: String,
    pub task_name: String,
}