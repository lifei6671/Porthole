use crate::app_state::AppState;
use crate::support::lifecycle::AppLifecycleState;
use tauri::Manager;

pub fn hide_to_tray_inner(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let Some(window) = app_handle.get_webview_window("main") else {
        return Err("未找到主窗口".to_string());
    };

    window.hide().map_err(|err| format!("隐藏主窗口失败: {err}"))
}

pub fn exit_application_inner(
    app_handle: &tauri::AppHandle,
    state: &AppState,
    lifecycle: &AppLifecycleState,
) -> Result<(), String> {
    if !lifecycle.begin_exit() {
        return Ok(());
    }

    hide_to_tray_inner(app_handle)?;
    state.prepare_for_exit();
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
pub fn hide_to_tray(app_handle: tauri::AppHandle) -> Result<(), String> {
    hide_to_tray_inner(&app_handle)
}

#[tauri::command]
pub fn exit_application(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    lifecycle: tauri::State<'_, AppLifecycleState>,
) -> Result<(), String> {
    exit_application_inner(&app_handle, &state, &lifecycle)
}
