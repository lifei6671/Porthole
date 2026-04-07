use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleProcessStatus {
    Stopped,
    Starting,
    Running,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleRuntimeStatus {
    pub rule_id: String,
    pub status: RuleProcessStatus,
    pub last_error_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeErrorSummary {
    pub summary: String,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeState {
    pub process_status: ProcessStatus,
    #[serde(default)]
    pub rule_statuses: Vec<RuleRuntimeStatus>,
    pub last_error: Option<RuntimeErrorSummary>,
    #[serde(default)]
    pub active_rule_ids: BTreeSet<String>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            process_status: ProcessStatus::Stopped,
            rule_statuses: Vec::new(),
            last_error: None,
            active_rule_ids: BTreeSet::new(),
        }
    }
}
