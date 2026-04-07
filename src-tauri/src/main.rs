#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app_state;
mod commands;
mod model;
mod service;
mod support {
    pub mod job_object;
    pub mod paths;
    pub mod pid_file;
}

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use app_state::AppState;
use commands::rules::{create_rule, delete_rule, list_rules, update_rule};
use commands::runtime::{
    clear_logs, get_runtime_status, start_all_enabled_rules, start_rule, stop_all_rules, stop_rule,
};
use service::gost_process::GostProcessManager;
use service::rule_store::RuleStore;
use service::runtime_events::{
    spawn_runtime_event_bridge, RuntimeEventEmitter, RuntimeEventMessage, EVENT_LOGS_APPENDED,
    EVENT_PROCESS_EXITED, EVENT_RULES_CHANGED, EVENT_RUNTIME_CHANGED,
};
use support::paths::AppPaths;
use tauri::{Emitter, Manager, WindowEvent};

fn main() {
    let exit_in_progress = Arc::new(AtomicBool::new(false));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .on_window_event({
            let exit_in_progress = Arc::clone(&exit_in_progress);
            move |window, event| {
                if window.label() != "main" {
                    return;
                }

                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();

                    if exit_in_progress.swap(true, Ordering::SeqCst) {
                        return;
                    }

                    let app_handle = window.app_handle().clone();
                    let state = app_handle.state::<AppState>().inner().clone();
                    let window = window.clone();

                    thread::spawn(move || {
                        let _ = window.hide();
                        state.prepare_for_exit();
                        app_handle.exit(0);
                    });
                }
            }
        })
        .setup(|app| {
            let paths = AppPaths::from_app_handle(app.handle())?;
            let rule_store = RuleStore::new(paths.clone());
            rule_store.ensure_initialized()?;
            let gost_process = GostProcessManager::new(paths.clone());
            let emitter = build_runtime_event_emitter(app.handle().clone());
            let state = AppState::new(
                paths.clone(),
                rule_store,
                gost_process.clone(),
                emitter.clone(),
                resolve_sidecar_path(app.handle()),
            );
            spawn_runtime_event_bridge(gost_process, emitter, Duration::from_millis(100));
            let restore_state = state.clone();
            app.manage(state);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(350));
                if let Err(err) = restore_state.restore_last_active_rules() {
                    restore_state.append_app_error_log(format!(
                        "恢复上次运行中的端口转发失败：{err}"
                    ));
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_rules,
            create_rule,
            update_rule,
            delete_rule,
            start_rule,
            stop_rule,
            start_all_enabled_rules,
            stop_all_rules,
            get_runtime_status,
            clear_logs
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}

fn build_runtime_event_emitter(app_handle: tauri::AppHandle) -> RuntimeEventEmitter {
    RuntimeEventEmitter::new(std::sync::Arc::new(move |message| match message {
        RuntimeEventMessage::RuntimeChanged(payload) => {
            let _ = app_handle.emit(EVENT_RUNTIME_CHANGED, payload);
        }
        RuntimeEventMessage::RulesChanged(payload) => {
            let _ = app_handle.emit(EVENT_RULES_CHANGED, payload);
        }
        RuntimeEventMessage::LogsAppended(payload) => {
            let _ = app_handle.emit(EVENT_LOGS_APPENDED, payload);
        }
        RuntimeEventMessage::ProcessExited(payload) => {
            let _ = app_handle.emit(EVENT_PROCESS_EXITED, payload);
        }
    }))
}

fn resolve_sidecar_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut candidates = Vec::new();

    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        candidates.push(resource_dir.join("gost.exe"));
        candidates.push(resource_dir.join("gost"));
        candidates.push(resource_dir.join("gost-x86_64-pc-windows-msvc.exe"));
        candidates.push(resource_dir.join("gost-x86_64-pc-windows-gnu.exe"));
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            candidates.push(parent.join("gost.exe"));
            candidates.push(parent.join("gost"));
            candidates.push(parent.join("gost-x86_64-pc-windows-msvc.exe"));
            candidates.push(parent.join("gost-x86_64-pc-windows-gnu.exe"));
        }
    }

    candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from("gost"))
}
