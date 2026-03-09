use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContestInfo {
    pub submit_url: String,
    pub language_id: String,
    pub problem_infos: Vec<ProblemInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProblemInfo {
    pub short_name: String,
    pub full_name: String,
}
