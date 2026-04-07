use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde::Serialize;

use crate::model::runtime::{
    ProcessStatus, RuleProcessStatus, RuleRuntimeStatus, RuntimeErrorSummary, RuntimeState,
};
use crate::support::job_object::JobObject;
use crate::support::paths::AppPaths;
use crate::support::pid_file::{clear_pid_file, read_pid_file, write_pid_file};

const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_millis(100);
const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_LOG_ENTRIES: usize = 500;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

type ProcessLauncher =
    dyn Fn(&GostLaunchRequest) -> Result<Child, GostProcessError> + Send + Sync + 'static;
type HealthChecker =
    dyn Fn(&str, Duration, Duration) -> Result<(), GostProcessError> + Send + Sync + 'static;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProcessLogEntry {
    pub source: ProcessLogSource,
    pub message: String,
    pub observed_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProcessLogSource {
    Stdout,
    Stderr,
    AppInfo,
    AppError,
}

#[derive(Debug, Clone)]
pub struct GostLaunchRequest {
    pub executable_path: PathBuf,
    pub args: Vec<String>,
    pub api_probe_url: String,
    pub active_rule_ids: BTreeSet<String>,
    pub health_check_interval: Duration,
    pub health_check_timeout: Duration,
}

impl GostLaunchRequest {
    pub fn new(
        executable_path: impl Into<PathBuf>,
        args: Vec<String>,
        api_probe_url: impl Into<String>,
        active_rule_ids: BTreeSet<String>,
    ) -> Self {
        Self {
            executable_path: executable_path.into(),
            args,
            api_probe_url: api_probe_url.into(),
            active_rule_ids,
            health_check_interval: DEFAULT_HEALTH_CHECK_INTERVAL,
            health_check_timeout: DEFAULT_HEALTH_CHECK_TIMEOUT,
        }
    }
}

#[derive(Debug)]
pub enum GostProcessError {
    Io(String, io::Error),
    HealthCheck(String),
    InvalidConfig(String),
}

impl Display for GostProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message, err) => write!(f, "{message}: {err}"),
            Self::HealthCheck(message) | Self::InvalidConfig(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for GostProcessError {}

#[derive(Clone)]
pub struct GostProcessManager {
    paths: AppPaths,
    state: Arc<Mutex<GostProcessState>>,
    operation_lock: Arc<Mutex<()>>,
    launcher: Arc<ProcessLauncher>,
    health_checker: Arc<HealthChecker>,
}

struct GostProcessState {
    runtime: RuntimeState,
    process: Option<ManagedProcess>,
    logs: Vec<ProcessLogEntry>,
}

struct ManagedProcess {
    child: Arc<Mutex<Child>>,
    pid: u32,
    _job: JobObject,
}

impl GostProcessManager {
    pub fn new(paths: AppPaths) -> Self {
        Self::new_with_hooks(
            paths,
            Arc::new(default_launcher),
            Arc::new(default_health_checker),
        )
    }

    pub fn new_with_hooks(
        paths: AppPaths,
        launcher: Arc<ProcessLauncher>,
        health_checker: Arc<HealthChecker>,
    ) -> Self {
        Self {
            paths,
            state: Arc::new(Mutex::new(GostProcessState {
                runtime: RuntimeState::default(),
                process: None,
                logs: Vec::new(),
            })),
            operation_lock: Arc::new(Mutex::new(())),
            launcher,
            health_checker,
        }
    }

    pub fn start(&self, request: GostLaunchRequest) -> Result<RuntimeState, GostProcessError> {
        let _guard = self.operation_lock.lock().map_err(|_| {
            GostProcessError::InvalidConfig("进程管理锁已损坏，无法启动 gost".to_string())
        })?;

        self.cleanup_stale_pid_file()?;
        self.start_locked(request)
    }

    pub fn stop(&self) -> Result<RuntimeState, GostProcessError> {
        let _guard = self.operation_lock.lock().map_err(|_| {
            GostProcessError::InvalidConfig("进程管理锁已损坏，无法停止 gost".to_string())
        })?;

        self.stop_locked(None)
    }

    pub fn reload(&self, request: GostLaunchRequest) -> Result<RuntimeState, GostProcessError> {
        let _guard = self.operation_lock.lock().map_err(|_| {
            GostProcessError::InvalidConfig("进程管理锁已损坏，无法重载 gost".to_string())
        })?;

        self.stop_locked(None)?;
        self.cleanup_stale_pid_file()?;
        self.start_locked(request)
    }

    pub fn runtime_snapshot(&self) -> RuntimeState {
        self.state
            .lock()
            .expect("gost process state poisoned")
            .runtime
            .clone()
    }

    pub fn log_snapshot(&self) -> Vec<ProcessLogEntry> {
        self.state
            .lock()
            .expect("gost process state poisoned")
            .logs
            .clone()
    }

    pub fn clear_logs(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.logs.clear();
        }
    }

    pub fn append_app_info_log(&self, message: impl Into<String>) {
        self.append_log(ProcessLogSource::AppInfo, message.into());
    }

    pub fn append_app_error_log(&self, message: impl Into<String>) {
        self.append_log(ProcessLogSource::AppError, message.into());
    }

    fn append_log(&self, source: ProcessLogSource, message: String) {
        if let Ok(mut state) = self.state.lock() {
            push_log(&mut state.logs, source, message);
        }
    }

    fn start_locked(&self, request: GostLaunchRequest) -> Result<RuntimeState, GostProcessError> {
        if request.active_rule_ids.is_empty() {
            return Err(GostProcessError::InvalidConfig(
                "运行集合为空，无法启动 gost".to_string(),
            ));
        }
        ensure_loopback_probe_url(&request.api_probe_url)?;

        self.paths
            .ensure_data_dir()
            .map_err(|err| GostProcessError::Io("创建应用数据目录失败".to_string(), err))?;

        let mut child = (self.launcher)(&request)?;
        let pid = child.id();
        let job = JobObject::new()
            .map_err(|err| GostProcessError::Io("创建 Job Object 失败".to_string(), err))?;
        job.attach_child(&child)
            .map_err(|err| GostProcessError::Io("绑定 Job Object 失败".to_string(), err))?;

        write_pid_file(self.paths.gost_pid_file(), pid)
            .map_err(|err| GostProcessError::Io("写入 gost PID 文件失败".to_string(), err))?;

        {
            let mut state = self.state.lock().expect("gost process state poisoned");
            state.runtime = RuntimeState {
                process_status: ProcessStatus::Starting,
                rule_statuses: build_rule_statuses(
                    &request.active_rule_ids,
                    RuleProcessStatus::Starting,
                    None,
                ),
                last_error: None,
                active_rule_ids: request.active_rule_ids.clone(),
            };
        }

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let child = Arc::new(Mutex::new(child));

        {
            let mut state = self.state.lock().expect("gost process state poisoned");
            state.process = Some(ManagedProcess {
                child: Arc::clone(&child),
                pid,
                _job: job,
            });
        }

        if let Some(stdout) = stdout {
            spawn_log_reader(Arc::clone(&self.state), stdout, ProcessLogSource::Stdout);
        }
        if let Some(stderr) = stderr {
            spawn_log_reader(Arc::clone(&self.state), stderr, ProcessLogSource::Stderr);
        }

        spawn_exit_watcher(
            Arc::clone(&self.state),
            self.paths.clone(),
            Arc::clone(&child),
            pid,
        );

        if let Err(err) = (self.health_checker)(
            &request.api_probe_url,
            request.health_check_interval,
            request.health_check_timeout,
        ) {
            let message = format!("gost API 探活失败: {err}");
            let _ = self.stop_locked(Some(message.clone()));
            return Err(GostProcessError::HealthCheck(message));
        }

        let mut state = self.state.lock().expect("gost process state poisoned");
        state.runtime.process_status = ProcessStatus::Running;
        update_rule_statuses(&mut state.runtime, RuleProcessStatus::Running, None);
        Ok(state.runtime.clone())
    }

    fn stop_locked(
        &self,
        failure_message: Option<String>,
    ) -> Result<RuntimeState, GostProcessError> {
        let managed = {
            let mut state = self.state.lock().expect("gost process state poisoned");
            if state.process.is_none() {
                if let Some(message) = failure_message.clone() {
                    state.runtime.process_status = ProcessStatus::Failed;
                    state.runtime.last_error = Some(RuntimeErrorSummary {
                        summary: message.clone(),
                        observed_at: Utc::now(),
                    });
                    update_rule_statuses(
                        &mut state.runtime,
                        RuleProcessStatus::Failed,
                        Some(message),
                    );
                } else {
                    state.runtime.process_status = ProcessStatus::Stopped;
                    state.runtime.active_rule_ids.clear();
                    update_rule_statuses(&mut state.runtime, RuleProcessStatus::Stopped, None);
                }
                return Ok(state.runtime.clone());
            }

            state.runtime.process_status = ProcessStatus::Stopping;
            state.process.take()
        };

        if let Some(managed) = managed {
            let child = Arc::clone(&managed.child);
            {
                let mut child = child.lock().expect("child mutex poisoned");
                let _ = child.kill();
            }

            wait_for_child_exit(&child, Duration::from_secs(2));
        }

        clear_pid_file(self.paths.gost_pid_file())
            .map_err(|err| GostProcessError::Io("清理 gost PID 文件失败".to_string(), err))?;

        let mut state = self.state.lock().expect("gost process state poisoned");
        state.process = None;

        if let Some(message) = failure_message {
            state.runtime.process_status = ProcessStatus::Failed;
            state.runtime.last_error = Some(RuntimeErrorSummary {
                summary: message.clone(),
                observed_at: Utc::now(),
            });
            update_rule_statuses(&mut state.runtime, RuleProcessStatus::Failed, Some(message));
        } else {
            state.runtime.process_status = ProcessStatus::Stopped;
            state.runtime.last_error = None;
            state.runtime.active_rule_ids.clear();
            update_rule_statuses(&mut state.runtime, RuleProcessStatus::Stopped, None);
        }

        Ok(state.runtime.clone())
    }

    fn cleanup_stale_pid_file(&self) -> Result<(), GostProcessError> {
        match read_pid_file(self.paths.gost_pid_file()) {
            Ok(Some(pid)) => {
                let _ = terminate_process(pid);
                clear_pid_file(self.paths.gost_pid_file()).map_err(|err| {
                    GostProcessError::Io("清理残留 PID 文件失败".to_string(), err)
                })?;
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(_) => {
                clear_pid_file(self.paths.gost_pid_file()).map_err(|err| {
                    GostProcessError::Io("清理损坏 PID 文件失败".to_string(), err)
                })?;
                Ok(())
            }
        }
    }
}

fn default_launcher(request: &GostLaunchRequest) -> Result<Child, GostProcessError> {
    let mut command = Command::new(&request.executable_path);
    command
        .args(&request.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        command.creation_flags(CREATE_NO_WINDOW);
    }

    command.spawn().map_err(|err| {
        GostProcessError::Io(
            format!(
                "启动 gost 进程失败: {} {:?}",
                request.executable_path.display(),
                request.args
            ),
            err,
        )
    })
}

fn default_health_checker(
    url: &str,
    interval: Duration,
    timeout: Duration,
) -> Result<(), GostProcessError> {
    let target = parse_http_probe_url(url)?;
    let deadline = Instant::now() + timeout;

    loop {
        match probe_http_endpoint(&target, interval) {
            Ok(()) => return Ok(()),
            Err(last_error) => {
                if Instant::now() >= deadline {
                    return Err(GostProcessError::HealthCheck(format!(
                        "探活超时，URL={url}，最后一次错误: {last_error}"
                    )));
                }
            }
        }

        thread::sleep(interval);
    }
}

fn ensure_loopback_probe_url(url: &str) -> Result<(), GostProcessError> {
    let target = parse_http_probe_url(url)?;
    match target.host_header.as_str() {
        "127.0.0.1" | "::1" => Ok(()),
        host => Err(GostProcessError::InvalidConfig(format!(
            "gost API 探活地址必须绑定本地回环地址，当前为: {host}"
        ))),
    }
}

#[derive(Debug, Clone)]
struct HttpProbeTarget {
    connect_addr: String,
    host_header: String,
    path: String,
}

fn parse_http_probe_url(url: &str) -> Result<HttpProbeTarget, GostProcessError> {
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        GostProcessError::InvalidConfig(format!("API 探活地址不是合法的 HTTP URL: {url}"))
    })?;

    let (authority, path) = match rest.split_once('/') {
        Some((authority, suffix)) => (authority, format!("/{suffix}")),
        None => (rest, "/".to_string()),
    };

    if authority.is_empty() {
        return Err(GostProcessError::InvalidConfig(format!(
            "API 探活地址缺少主机信息: {url}"
        )));
    }

    let (host, port) = if authority.starts_with('[') {
        let end = authority.find(']').ok_or_else(|| {
            GostProcessError::InvalidConfig(format!("API 探活地址中的 IPv6 格式非法: {url}"))
        })?;
        let host = &authority[1..end];
        let port = authority[end + 1..]
            .strip_prefix(':')
            .ok_or_else(|| GostProcessError::InvalidConfig(format!("API 探活地址缺少端口: {url}")))?
            .parse::<u16>()
            .map_err(|err| GostProcessError::InvalidConfig(format!("API 探活端口非法: {err}")))?;
        (host.to_string(), port)
    } else {
        let (host, port) = authority.rsplit_once(':').ok_or_else(|| {
            GostProcessError::InvalidConfig(format!("API 探活地址缺少端口: {url}"))
        })?;
        let port = port
            .parse::<u16>()
            .map_err(|err| GostProcessError::InvalidConfig(format!("API 探活端口非法: {err}")))?;
        (host.to_string(), port)
    };

    let connect_addr = if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };

    Ok(HttpProbeTarget {
        connect_addr,
        host_header: host,
        path,
    })
}

fn probe_http_endpoint(target: &HttpProbeTarget, interval: Duration) -> Result<(), io::Error> {
    let mut addrs = target.connect_addr.to_socket_addrs()?;
    let addr = addrs.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::AddrNotAvailable, "探活地址没有可用解析结果")
    })?;

    let mut stream = TcpStream::connect_timeout(&addr, interval)?;
    stream.set_read_timeout(Some(interval))?;
    stream.set_write_timeout(Some(interval))?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        target.path, target.host_header
    );
    stream.write_all(request.as_bytes())?;

    let mut buffer = [0_u8; 64];
    let bytes = stream.read(&mut buffer)?;
    if bytes == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "探活请求没有收到任何响应",
        ));
    }

    Ok(())
}

fn spawn_log_reader<R>(state: Arc<Mutex<GostProcessState>>, reader: R, source: ProcessLogSource)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(_) => break,
            };

            let mut state = match state.lock() {
                Ok(state) => state,
                Err(_) => break,
            };
            push_log(&mut state.logs, source, line.clone());
            if matches!(source, ProcessLogSource::Stderr) {
                state.runtime.last_error = Some(RuntimeErrorSummary {
                    summary: line.clone(),
                    observed_at: Utc::now(),
                });
                if matches!(state.runtime.process_status, ProcessStatus::Running) {
                    update_rule_statuses(
                        &mut state.runtime,
                        RuleProcessStatus::Running,
                        Some(line),
                    );
                }
            }
        }
    });
}

fn spawn_exit_watcher(
    state: Arc<Mutex<GostProcessState>>,
    paths: AppPaths,
    child: Arc<Mutex<Child>>,
    pid: u32,
) {
    thread::spawn(move || loop {
        let exit_status = {
            let mut child = match child.lock() {
                Ok(child) => child,
                Err(_) => return,
            };
            match child.try_wait() {
                Ok(Some(status)) => status,
                Ok(None) => {
                    drop(child);
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(_) => return,
            }
        };

        let _ = clear_pid_file(paths.gost_pid_file());
        let mut state = match state.lock() {
            Ok(state) => state,
            Err(_) => return,
        };

        let still_same_process = state
            .process
            .as_ref()
            .map(|process| process.pid == pid)
            .unwrap_or(false);
        if !still_same_process {
            return;
        }

        state.process = None;
        state.runtime.active_rule_ids.clear();

        if matches!(state.runtime.process_status, ProcessStatus::Stopping) {
            state.runtime.process_status = ProcessStatus::Stopped;
            state.runtime.last_error = None;
            update_rule_statuses(&mut state.runtime, RuleProcessStatus::Stopped, None);
            return;
        }

        if exit_status.success() {
            state.runtime.process_status = ProcessStatus::Stopped;
            update_rule_statuses(&mut state.runtime, RuleProcessStatus::Stopped, None);
            return;
        }

        let message = match exit_status.code() {
            Some(code) => format!("gost 进程异常退出，退出码: {code}"),
            None => "gost 进程异常退出，未获取到退出码".to_string(),
        };
        state.runtime.process_status = ProcessStatus::Failed;
        state.runtime.last_error = Some(RuntimeErrorSummary {
            summary: message.clone(),
            observed_at: Utc::now(),
        });
        update_rule_statuses(&mut state.runtime, RuleProcessStatus::Failed, Some(message));
        return;
    });
}

fn wait_for_child_exit(child: &Arc<Mutex<Child>>, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        let exited = {
            let mut child = match child.lock() {
                Ok(child) => child,
                Err(_) => return,
            };
            matches!(child.try_wait(), Ok(Some(_)))
        };

        if exited || Instant::now() >= deadline {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn build_rule_statuses(
    rule_ids: &BTreeSet<String>,
    status: RuleProcessStatus,
    error: Option<String>,
) -> Vec<RuleRuntimeStatus> {
    rule_ids
        .iter()
        .map(|rule_id| RuleRuntimeStatus {
            rule_id: rule_id.clone(),
            status,
            last_error_summary: error.clone(),
        })
        .collect()
}

fn update_rule_statuses(
    runtime: &mut RuntimeState,
    status: RuleProcessStatus,
    error: Option<String>,
) {
    for rule_status in &mut runtime.rule_statuses {
        rule_status.status = status;
        rule_status.last_error_summary = error.clone();
    }
}

fn push_log(logs: &mut Vec<ProcessLogEntry>, source: ProcessLogSource, message: String) {
    logs.push(ProcessLogEntry {
        source,
        message,
        observed_at: Utc::now(),
    });
    if logs.len() > MAX_LOG_ENTRIES {
        let excess = logs.len() - MAX_LOG_ENTRIES;
        logs.drain(0..excess);
    }
}

#[cfg(windows)]
fn terminate_process(pid: u32) -> io::Result<()> {
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F", "/T"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("taskkill 退出码异常: {status}"),
        ))
    }
}

#[cfg(not(windows))]
fn terminate_process(pid: u32) -> io::Result<()> {
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("kill 退出码异常: {status}"),
        ))
    }
}
