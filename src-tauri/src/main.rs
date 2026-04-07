mod app_state;
mod model;
mod service;
mod support {
    pub mod paths;
}

use app_state::AppState;
use service::rule_store::RuleStore;
use support::paths::AppPaths;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let paths = AppPaths::from_app_handle(app.handle())?;
            let rule_store = RuleStore::new(paths.clone());
            rule_store.ensure_initialized()?;
            app.manage(AppState::new(paths, rule_store));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}
