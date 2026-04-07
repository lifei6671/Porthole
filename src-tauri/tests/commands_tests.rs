#[path = "../src/app_state.rs"]
mod app_state;

#[path = "../src/commands/mod.rs"]
mod commands;

#[path = "../src/model/mod.rs"]
mod model;

#[path = "../src/support/paths.rs"]
pub mod support_paths;

#[path = "../src/support/job_object.rs"]
pub mod support_job_object;

#[path = "../src/support/pid_file.rs"]
pub mod support_pid_file;

#[path = "../src/support/lifecycle.rs"]
pub mod support_lifecycle;

mod support {
    pub use crate::support_job_object as job_object;
    pub use crate::support_lifecycle as lifecycle;
    pub use crate::support_paths as paths;
    pub use crate::support_pid_file as pid_file;
}

#[path = "../src/service/mod.rs"]
mod service;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use app_state::AppState;
use commands::rules::{
    create_rule_inner, delete_rule_inner, list_rules_inner, update_rule_inner, RuleInputPayload,
};
use commands::runtime::{
    clear_logs_inner, get_runtime_status_inner, start_all_enabled_rules_inner, start_rule_inner,
    stop_all_rules_inner,
};
use model::rule::Protocol;
use service::gost_process::{GostProcessError, GostProcessManager};
use service::rule_store::RuleStore;
use service::runtime_events::{
    spawn_runtime_event_bridge, RuntimeEventEmitter, RuntimeEventMessage,
};
use support::paths::AppPaths;
use tempfile::tempdir;

#[cfg(unix)]
fn spawn_script_child(script: &str) -> Child {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().expect("spawn test child")
}

#[cfg(unix)]
fn sample_rule_input(name: &str, enabled: bool) -> RuleInputPayload {
    RuleInputPayload {
        name: name.to_string(),
        enabled,
        protocol: Protocol::Tcp,
        listen_host: "127.0.0.1".to_string(),
        listen_port: 28080,
        target_host: "10.0.0.10".to_string(),
        target_port: 80,
        remark: "test".to_string(),
    }
}

#[cfg(unix)]
fn sleep_launcher(_: &service::gost_process::GostLaunchRequest) -> Result<Child, GostProcessError> {
    Ok(spawn_script_child("sleep 1"))
}

#[cfg(unix)]
fn build_state(
    launcher: Arc<
        dyn Fn(&service::gost_process::GostLaunchRequest) -> Result<Child, GostProcessError>
            + Send
            + Sync
            + 'static,
    >,
) -> (AppState, Arc<Mutex<Vec<RuntimeEventMessage>>>) {
    let temp_dir = tempdir().expect("create temp dir");
    let temp_path = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir);
    build_state_with_path(temp_path, launcher)
}

#[cfg(unix)]
fn build_state_with_path(
    temp_path: PathBuf,
    launcher: Arc<
        dyn Fn(&service::gost_process::GostLaunchRequest) -> Result<Child, GostProcessError>
            + Send
            + Sync
            + 'static,
    >,
) -> (AppState, Arc<Mutex<Vec<RuntimeEventMessage>>>) {
    let paths = AppPaths::new(temp_path);
    let rule_store = RuleStore::new(paths.clone());
    rule_store.ensure_initialized().expect("init rule store");

    let events = Arc::new(Mutex::new(Vec::new()));
    let emitter = RuntimeEventEmitter::new(Arc::new({
        let events = Arc::clone(&events);
        move |message| {
            events.lock().expect("event lock poisoned").push(message);
        }
    }));

    let manager =
        GostProcessManager::new_with_hooks(paths.clone(), launcher, Arc::new(|_, _, _| Ok(())));

    let state = AppState::new_with_api_binding_provider(
        paths,
        rule_store,
        manager.clone(),
        emitter.clone(),
        PathBuf::from("/bin/sh"),
        Arc::new(|| {
            Ok(app_state::GostApiBinding {
                listen_addr: "127.0.0.1:24680?pathPrefix=/api".to_string(),
                probe_url: "http://127.0.0.1:24680/api/config/services".to_string(),
            })
        }),
    );

    spawn_runtime_event_bridge(manager, emitter, Duration::from_millis(50));
    (state, events)
}

#[cfg(unix)]
fn wait_until(description: &str, timeout: Duration, predicate: impl Fn() -> bool) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if predicate() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }

    panic!("timed out while waiting for {description}");
}

#[cfg(unix)]
#[test]
fn rule_commands_round_trip_and_emit_rules_changed() {
    let (state, events) = build_state(Arc::new(|_| Ok(spawn_script_child("sleep 2"))));

    let created =
        create_rule_inner(&state, sample_rule_input("HTTP Forward", true)).expect("create rule");
    assert_eq!(created.name, "HTTP Forward");

    let listed = list_rules_inner(&state).expect("list rules");
    assert_eq!(listed.len(), 1);

    let updated = update_rule_inner(
        &state,
        &created.id,
        RuleInputPayload {
            name: "Updated Forward".to_string(),
            listen_port: 28081,
            target_port: 8080,
            ..sample_rule_input("ignored", false)
        },
    )
    .expect("update rule");
    assert_eq!(updated.name, "Updated Forward");
    assert!(!updated.enabled);

    delete_rule_inner(&state, &created.id).expect("delete rule");
    assert!(list_rules_inner(&state)
        .expect("list after delete")
        .is_empty());

    let recorded = events.lock().expect("event lock poisoned");
    let payloads = recorded
        .iter()
        .filter_map(|event| match event {
            RuntimeEventMessage::RulesChanged(payload) => Some(payload),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(payloads.len() >= 3);
    assert_eq!(payloads.first().expect("first payload").rules.len(), 1);
    assert_eq!(payloads.last().expect("last payload").rules.len(), 0);
}

#[cfg(unix)]
#[test]
fn runtime_commands_return_snapshot_and_support_log_clearing() {
    let (state, events) = build_state(Arc::new(|_| {
        Ok(spawn_script_child(
            "echo hello-from-stdout; echo hello-from-stderr 1>&2; sleep 3",
        ))
    }));

    let created = create_rule_inner(&state, sample_rule_input("Runtime Rule", true))
        .expect("create runtime rule");
    let runtime = start_all_enabled_rules_inner(&state).expect("start all enabled");
    assert!(matches!(
        runtime.process_status,
        model::runtime::ProcessStatus::Running
    ));
    assert!(runtime.active_rule_ids.contains(&created.id));

    wait_until("logs to be captured", Duration::from_secs(2), || {
        !get_runtime_status_inner(&state).logs.is_empty()
    });

    let snapshot = get_runtime_status_inner(&state);
    assert!(matches!(
        snapshot.runtime.process_status,
        model::runtime::ProcessStatus::Running
    ));
    assert!(snapshot
        .logs
        .iter()
        .any(|entry| entry.message.contains("hello-from-stdout")));

    clear_logs_inner(&state);
    assert!(get_runtime_status_inner(&state).logs.is_empty());

    let stopped = stop_all_rules_inner(&state).expect("stop all");
    assert!(matches!(
        stopped.process_status,
        model::runtime::ProcessStatus::Stopped
    ));

    let recorded = events.lock().expect("event lock poisoned");
    assert!(recorded.iter().any(|event| matches!(
        event,
        RuntimeEventMessage::RuntimeChanged(payload)
            if matches!(payload.runtime.process_status, model::runtime::ProcessStatus::Running)
    )));
}

#[cfg(unix)]
#[test]
fn runtime_bridge_emits_process_exit_and_log_events() {
    let (state, events) = build_state(Arc::new(|_| {
        Ok(spawn_script_child(
            "echo process-started; echo process-error 1>&2; sleep 0.2",
        ))
    }));

    let created =
        create_rule_inner(&state, sample_rule_input("Exit Rule", true)).expect("create exit rule");
    start_rule_inner(&state, &created.id).expect("start single rule");

    wait_until("process exited event", Duration::from_secs(3), || {
        events
            .lock()
            .expect("event lock poisoned")
            .iter()
            .any(|event| matches!(event, RuntimeEventMessage::ProcessExited(_)))
    });

    let recorded = events.lock().expect("event lock poisoned");
    assert!(recorded.iter().any(|event| matches!(
        event,
        RuntimeEventMessage::LogsAppended(payload)
            if payload
                .entries
                .iter()
                .any(|entry| entry.message.contains("process-started"))
    )));
    assert!(recorded.iter().any(|event| matches!(
        event,
        RuntimeEventMessage::ProcessExited(payload)
            if matches!(
                payload.runtime.process_status,
                model::runtime::ProcessStatus::Stopped | model::runtime::ProcessStatus::Failed
            )
    )));

    let snapshot = get_runtime_status_inner(&state);
    assert!(matches!(
        snapshot.runtime.process_status,
        model::runtime::ProcessStatus::Stopped
    ));
}

#[cfg(unix)]
#[test]
fn app_state_restores_last_active_rules_on_next_boot() {
    let temp_dir = tempdir().expect("create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    let launcher = Arc::new(sleep_launcher);
    let (state, _) = build_state_with_path(temp_path.clone(), launcher.clone());

    let created =
        create_rule_inner(&state, sample_rule_input("Restore Rule", true)).expect("create rule");
    start_rule_inner(&state, &created.id).expect("start rule");

    let (restored_state, _) = build_state_with_path(temp_path, launcher);
    let restored = restored_state
        .restore_last_active_rules()
        .expect("restore last active rules")
        .expect("runtime should be restored");

    assert!(matches!(
        restored.process_status,
        model::runtime::ProcessStatus::Running
    ));
    assert!(restored.active_rule_ids.contains(&created.id));
}
