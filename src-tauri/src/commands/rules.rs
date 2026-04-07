use serde::{Deserialize, Serialize};

use crate::app_state::{AppState, AppStateError, RuleInput};
use crate::model::rule::{Protocol, Rule};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleInputPayload {
    pub name: String,
    pub enabled: bool,
    pub protocol: Protocol,
    pub listen_host: String,
    pub listen_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub remark: String,
}

impl From<RuleInputPayload> for RuleInput {
    fn from(value: RuleInputPayload) -> Self {
        Self {
            name: value.name,
            enabled: value.enabled,
            protocol: value.protocol,
            listen_host: value.listen_host,
            listen_port: value.listen_port,
            target_host: value.target_host,
            target_port: value.target_port,
            remark: value.remark,
        }
    }
}

pub fn list_rules_inner(state: &AppState) -> Result<Vec<Rule>, String> {
    state.list_rules().map_err(error_to_string)
}

pub fn create_rule_inner(state: &AppState, input: RuleInputPayload) -> Result<Rule, String> {
    state.create_rule(input.into()).map_err(error_to_string)
}

pub fn update_rule_inner(
    state: &AppState,
    rule_id: &str,
    input: RuleInputPayload,
) -> Result<Rule, String> {
    state
        .update_rule(rule_id, input.into())
        .map_err(error_to_string)
}

pub fn delete_rule_inner(state: &AppState, rule_id: &str) -> Result<(), String> {
    state.delete_rule(rule_id).map_err(error_to_string)
}

#[tauri::command]
pub fn list_rules(state: tauri::State<'_, AppState>) -> Result<Vec<Rule>, String> {
    list_rules_inner(&state)
}

#[tauri::command]
pub fn create_rule(
    state: tauri::State<'_, AppState>,
    input: RuleInputPayload,
) -> Result<Rule, String> {
    create_rule_inner(&state, input)
}

#[tauri::command]
pub fn update_rule(
    state: tauri::State<'_, AppState>,
    rule_id: String,
    input: RuleInputPayload,
) -> Result<Rule, String> {
    update_rule_inner(&state, &rule_id, input)
}

#[tauri::command]
pub fn delete_rule(state: tauri::State<'_, AppState>, rule_id: String) -> Result<(), String> {
    delete_rule_inner(&state, &rule_id)
}

fn error_to_string(err: AppStateError) -> String {
    err.to_string()
}
