#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app_state;
mod commands;
mod model;
mod service;
mod support {
    pub mod job_object;
    pub mod lifecycle;
    pub mod paths;
    pub mod pid_file;
}

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use app_state::AppState;
use commands::app::{exit_application, exit_application_inner, hide_to_tray};
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
use support::lifecycle::AppLifecycleState;
use support::paths::AppPaths;
use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, WindowEvent};

const EVENT_APP_CLOSE_REQUESTED: &str = "app://close-requested";
const TRAY_MENU_SHOW_MAIN: &str = "tray_show_main";
const TRAY_MENU_EXIT_APP: &str = "tray_exit_app";

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = show_main_window(app);
        }))
        .on_window_event({
            move |window, event| {
                if window.label() != "main" {
                    return;
                }

                if let WindowEvent::CloseRequested { api, .. } = event {
                    let lifecycle = window.app_handle().state::<AppLifecycleState>();
                    if lifecycle.is_exit_in_progress() {
                        return;
                    }

                    api.prevent_close();
                    let _ = window.emit(EVENT_APP_CLOSE_REQUESTED, ());
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
            let lifecycle = AppLifecycleState::default();
            app.manage(state);
            app.manage(lifecycle);
            setup_tray(app)?;
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
            clear_logs,
            hide_to_tray,
            exit_application
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

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let show_main = MenuItem::with_id(app.handle(), TRAY_MENU_SHOW_MAIN, "显示主窗口", true, None::<&str>)?;
    let exit_app = MenuItem::with_id(app.handle(), TRAY_MENU_EXIT_APP, "退出应用", true, None::<&str>)?;
    let menu = Menu::with_items(app.handle(), &[&show_main, &exit_app])?;
    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("Porthole")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event: MenuEvent| match event.id().as_ref() {
            TRAY_MENU_SHOW_MAIN => {
                let _ = show_main_window(app);
            }
            TRAY_MENU_EXIT_APP => {
                let _ = exit_application_inner(
                    app,
                    app.state::<AppState>().inner(),
                    app.state::<AppLifecycleState>().inner(),
                );
            }
            _ => {}
        })
        .on_tray_icon_event(|tray: &TrayIcon<_>, event| {
            if matches_left_click_release(&event) || matches_left_double_click(&event) {
                let _ = show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    let _ = builder.build(app.handle())?;
    Ok(())
}

fn show_main_window(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let Some(window) = app_handle.get_webview_window("main") else {
        return Err("未找到主窗口".to_string());
    };

    let _ = window.unminimize();
    window.show().map_err(|err| format!("显示主窗口失败: {err}"))?;
    window.set_focus().map_err(|err| format!("激活主窗口失败: {err}"))?;
    Ok(())
}

fn matches_left_click_release(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
    )
}

fn matches_left_double_click(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        }
    )
}
