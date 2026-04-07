#[path = "../src/model/mod.rs"]
mod model;

#[path = "../src/support/paths.rs"]
pub mod support_paths;

#[path = "../src/support/job_object.rs"]
pub mod support_job_object;

#[path = "../src/support/pid_file.rs"]
pub mod support_pid_file;

mod support {
    pub use crate::support_job_object as job_object;
    pub use crate::support_paths as paths;
    pub use crate::support_pid_file as pid_file;
}

#[path = "../src/service/mod.rs"]
mod service;

use std::collections::BTreeSet;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use service::gost_process::{GostLaunchRequest, GostProcessError, GostProcessManager};
use support::paths::AppPaths;
use tempfile::tempdir;

#[cfg(unix)]
fn spawn_sleep_child(seconds: &str) -> Child {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("sleep {seconds}"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().expect("spawn sleep child")
}

#[cfg(unix)]
fn spawn_noisy_child() -> Child {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg("echo hello-from-stdout; echo hello-from-stderr 1>&2; sleep 1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().expect("spawn noisy child")
}

#[cfg(unix)]
#[test]
fn child_exit_updates_runtime_status_and_captures_logs() {
    let temp_dir = tempdir().expect("create temp dir");
    let manager = GostProcessManager::new_with_hooks(
        AppPaths::new(temp_dir.path()),
        Arc::new(|_| {
            let child = spawn_noisy_child();
            Ok(child)
        }),
        Arc::new(|_, _, _| Ok(())),
    );

    let request = GostLaunchRequest::new(
        "/bin/sh",
        vec![],
        "http://127.0.0.1:19000/api/config/services",
        BTreeSet::from(["rule-1".to_string()]),
    );

    let runtime = manager.start(request).expect("start process");
    assert!(matches!(
        runtime.process_status,
        model::runtime::ProcessStatus::Running
    ));

    thread::sleep(Duration::from_millis(1200));

    let runtime = manager.runtime_snapshot();
    assert!(matches!(
        runtime.process_status,
        model::runtime::ProcessStatus::Stopped
    ));
    assert!(manager
        .log_snapshot()
        .iter()
        .any(|entry| entry.message.contains("hello-from-stdout")));
    assert!(manager
        .log_snapshot()
        .iter()
        .any(|entry| entry.message.contains("hello-from-stderr")));
}

#[cfg(unix)]
#[test]
fn stop_and_start_cleanup_pid_file() {
    let temp_dir = tempdir().expect("create temp dir");
    let paths = AppPaths::new(temp_dir.path());
    let manager = GostProcessManager::new_with_hooks(
        paths.clone(),
        Arc::new(|_| {
            let child = spawn_sleep_child("2");
            Ok(child)
        }),
        Arc::new(|_, _, _| Ok(())),
    );

    let request = GostLaunchRequest::new(
        "/bin/sh",
        vec![],
        "http://127.0.0.1:19001/api/config/services",
        BTreeSet::from(["rule-1".to_string()]),
    );

    manager.start(request).expect("start process");
    assert!(paths.gost_pid_file().exists());

    manager.stop().expect("stop process");
    assert!(!paths.gost_pid_file().exists());

    std::fs::write(paths.gost_pid_file(), "999999").expect("write stale pid");
    let request = GostLaunchRequest::new(
        "/bin/sh",
        vec![],
        "http://127.0.0.1:19002/api/config/services",
        BTreeSet::from(["rule-2".to_string()]),
    );
    manager.start(request).expect("restart process");
    let pid = std::fs::read_to_string(paths.gost_pid_file()).expect("read new pid");
    assert_ne!(pid.trim(), "999999");

    manager.stop().expect("final stop");
}

#[cfg(unix)]
#[test]
fn api_unreachable_is_reported_as_start_failure() {
    let temp_dir = tempdir().expect("create temp dir");
    let paths = AppPaths::new(temp_dir.path());
    let manager = GostProcessManager::new_with_hooks(
        paths.clone(),
        Arc::new(|_| {
            let child = spawn_sleep_child("3");
            Ok(child)
        }),
        Arc::new(|_, _, _| Err(GostProcessError::HealthCheck("模拟 API 不可达".to_string()))),
    );

    let mut request = GostLaunchRequest::new(
        "/bin/sh",
        vec![],
        "http://127.0.0.1:65530/api/config/services",
        BTreeSet::from(["rule-1".to_string()]),
    );
    request.health_check_timeout = Duration::from_millis(200);

    let error = manager.start(request).expect_err("start should fail");
    assert!(error.to_string().contains("API 探活失败"));
    assert!(!paths.gost_pid_file().exists());

    let runtime = manager.runtime_snapshot();
    assert!(matches!(
        runtime.process_status,
        model::runtime::ProcessStatus::Failed
    ));
}

#[cfg(unix)]
#[test]
fn non_loopback_probe_url_is_rejected_before_start() {
    let temp_dir = tempdir().expect("create temp dir");
    let manager = GostProcessManager::new_with_hooks(
        AppPaths::new(temp_dir.path()),
        Arc::new(|_| {
            panic!("launcher should not be called for non-loopback probe url");
        }),
        Arc::new(|_, _, _| Ok(())),
    );

    let request = GostLaunchRequest::new(
        "/bin/sh",
        vec![],
        "http://10.0.0.1:8080/api/config/services",
        BTreeSet::from(["rule-1".to_string()]),
    );

    let error = manager
        .start(request)
        .expect_err("non-loopback url should fail");
    assert!(error
        .to_string()
        .contains("gost API 探活地址必须绑定本地回环地址"));
}
