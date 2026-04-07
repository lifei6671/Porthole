use serde::Serialize;

use crate::app_state::{AppState, AppStateError};
use crate::model::runtime::RuntimeState;
use crate::service::gost_process::ProcessLogEntry;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStatusPayload {
    pub runtime: RuntimeState,
    pub logs: Vec<ProcessLogEntry>,
}

pub fn start_rule_inner(state: &AppState, rule_id: &str) -> Result<RuntimeState, String> {
    state.start_rule(rule_id).map_err(error_to_string)
}

pub fn stop_rule_inner(state: &AppState, rule_id: &str) -> Result<RuntimeState, String> {
    state.stop_rule(rule_id).map_err(error_to_string)
}

pub fn start_all_enabled_rules_inner(state: &AppState) -> Result<RuntimeState, String> {
    state.start_all_enabled_rules().map_err(error_to_string)
}

pub fn stop_all_rules_inner(state: &AppState) -> Result<RuntimeState, String> {
    state.stop_all_rules().map_err(error_to_string)
}

pub fn get_runtime_status_inner(state: &AppState) -> RuntimeStatusPayload {
    RuntimeStatusPayload {
        runtime: state.runtime_snapshot(),
        logs: state.log_snapshot(),
    }
}

pub fn clear_logs_inner(state: &AppState) {
    state.clear_logs();
}

#[tauri::command]
pub fn start_rule(
    state: tauri::State<'_, AppState>,
    rule_id: String,
) -> Result<RuntimeState, String> {
    start_rule_inner(&state, &rule_id)
}

#[tauri::command]
pub fn stop_rule(
    state: tauri::State<'_, AppState>,
    rule_id: String,
) -> Result<RuntimeState, String> {
    stop_rule_inner(&state, &rule_id)
}

#[tauri::command]
pub fn start_all_enabled_rules(state: tauri::State<'_, AppState>) -> Result<RuntimeState, String> {
    start_all_enabled_rules_inner(&state)
}

#[tauri::command]
pub fn stop_all_rules(state: tauri::State<'_, AppState>) -> Result<RuntimeState, String> {
    stop_all_rules_inner(&state)
}

#[tauri::command]
pub fn get_runtime_status(state: tauri::State<'_, AppState>) -> RuntimeStatusPayload {
    get_runtime_status_inner(&state)
}

#[tauri::command]
pub fn clear_logs(state: tauri::State<'_, AppState>) {
    clear_logs_inner(&state)
}

fn error_to_string(err: AppStateError) -> String {
    err.to_string()
}
