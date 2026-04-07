use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use crate::model::rule::{Protocol, Rule};
use crate::support::paths::AppPaths;

#[cfg(target_os = "windows")]
use std::fs;
#[cfg(target_os = "windows")]
use std::process::{Command, Stdio};

type FirewallRunner =
    dyn Fn(&FirewallExecution) -> Result<(), FirewallError> + Send + Sync + 'static;
type ElevationChecker = dyn Fn() -> Result<bool, FirewallError> + Send + Sync + 'static;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FirewallRuleSpec {
    pub display_name: String,
    pub protocol: Protocol,
    pub local_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FirewallSyncPlan {
    pub add_rules: Vec<FirewallRuleSpec>,
    pub remove_rule_names: Vec<String>,
}

impl FirewallSyncPlan {
    pub fn is_empty(&self) -> bool {
        self.add_rules.is_empty() && self.remove_rule_names.is_empty()
    }

    #[cfg(any(test, target_os = "windows"))]
    pub fn summary_messages(&self) -> Vec<String> {
        let mut messages = Vec::new();

        if !self.add_rules.is_empty() {
            let summary = self
                .add_rules
                .iter()
                .map(|rule| format!("{} {}", rule.protocol.as_str().to_uppercase(), rule.local_port))
                .collect::<Vec<_>>()
                .join(", ");
            messages.push(format!("已同步 Windows 防火墙放行规则：{summary}"));
        }

        if !self.remove_rule_names.is_empty() {
            messages.push(format!(
                "已清理 {} 条 Windows 防火墙规则",
                self.remove_rule_names.len()
            ));
        }

        messages
    }

    #[cfg(any(test, target_os = "windows"))]
    pub fn to_powershell_script(&self) -> String {
        let mut script = String::from(
            "$ErrorActionPreference = 'Stop'\nSet-StrictMode -Version Latest\n",
        );

        for rule_name in &self.remove_rule_names {
            let escaped = escape_powershell_single_quoted(rule_name);
            script.push_str(&format!(
                "Get-NetFirewallRule -DisplayName '{escaped}' -ErrorAction SilentlyContinue | Remove-NetFirewallRule -ErrorAction SilentlyContinue | Out-Null\n"
            ));
        }

        for rule in &self.add_rules {
            let escaped_name = escape_powershell_single_quoted(&rule.display_name);
            let protocol = match rule.protocol {
                Protocol::Tcp => "TCP",
                Protocol::Udp => "UDP",
            };
            script.push_str(&format!(
                "Get-NetFirewallRule -DisplayName '{escaped_name}' -ErrorAction SilentlyContinue | Remove-NetFirewallRule -ErrorAction SilentlyContinue | Out-Null\n"
            ));
            script.push_str(&format!(
                "New-NetFirewallRule -DisplayName '{escaped_name}' -Group 'Porthole' -Direction Inbound -Action Allow -Protocol {protocol} -LocalPort {} -Profile Any | Out-Null\n",
                rule.local_port
            ));
        }

        script
    }
}

#[derive(Debug, Clone)]
struct FirewallExecution {
    script_path: PathBuf,
    requires_elevation: bool,
}

#[derive(Debug)]
pub enum FirewallError {
    Io(String, io::Error),
    Command(String),
}

impl Display for FirewallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message, err) => write!(f, "{message}: {err}"),
            Self::Command(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for FirewallError {}

#[derive(Clone)]
pub struct FirewallManager {
    paths: AppPaths,
    runner: Arc<FirewallRunner>,
    elevation_checker: Arc<ElevationChecker>,
}

impl FirewallManager {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            runner: Arc::new(default_firewall_runner),
            elevation_checker: Arc::new(default_elevation_checker),
        }
    }

    pub fn sync_rules(
        &self,
        rules: &[Rule],
        active_rule_ids: &BTreeSet<String>,
    ) -> Result<Vec<String>, FirewallError> {
        let plan = build_sync_plan(rules, active_rule_ids);
        self.execute_plan(plan)
    }

    pub fn remove_rule(&self, rule: &Rule) -> Result<Vec<String>, FirewallError> {
        if !needs_firewall_rule(rule) {
            return Ok(Vec::new());
        }

        let plan = FirewallSyncPlan {
            add_rules: Vec::new(),
            remove_rule_names: vec![firewall_rule_name(rule)],
        };
        self.execute_plan(plan)
    }

    fn execute_plan(&self, plan: FirewallSyncPlan) -> Result<Vec<String>, FirewallError> {
        if plan.is_empty() {
            return Ok(Vec::new());
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = &self.paths;
            let _ = &self.runner;
            let _ = &self.elevation_checker;
            return Ok(Vec::new());
        }

        #[cfg(target_os = "windows")]
        {
            self.paths
                .ensure_data_dir()
                .map_err(|err| FirewallError::Io("创建应用数据目录失败".to_string(), err))?;

            let script_path = self.paths.data_dir().join("firewall-sync.ps1");
            fs::write(&script_path, plan.to_powershell_script()).map_err(|err| {
                FirewallError::Io("写入 Windows 防火墙脚本失败".to_string(), err)
            })?;

            let elevated = (self.elevation_checker)()?;
            let execution = FirewallExecution {
                script_path,
                requires_elevation: !elevated,
            };
            (self.runner)(&execution)?;
            Ok(plan.summary_messages())
        }
    }
}

pub(crate) fn build_sync_plan(
    rules: &[Rule],
    active_rule_ids: &BTreeSet<String>,
) -> FirewallSyncPlan {
    let mut add_rules = Vec::new();
    let mut remove_rule_names = Vec::new();

    for rule in rules {
        if !needs_firewall_rule(rule) {
            continue;
        }

        if active_rule_ids.contains(&rule.id) {
            add_rules.push(FirewallRuleSpec {
                display_name: firewall_rule_name(rule),
                protocol: rule.protocol,
                local_port: rule.listen_port,
            });
        } else {
            remove_rule_names.push(firewall_rule_name(rule));
        }
    }

    FirewallSyncPlan {
        add_rules,
        remove_rule_names,
    }
}

pub(crate) fn firewall_rule_name(rule: &Rule) -> String {
    format!(
        "Porthole-{}-{}-{}",
        rule.id,
        rule.protocol.as_str(),
        rule.listen_port
    )
}

pub(crate) fn needs_firewall_rule(rule: &Rule) -> bool {
    !is_loopback_host(&rule.listen_host)
}

pub(crate) fn is_loopback_host(host: &str) -> bool {
    matches!(host.trim(), "127.0.0.1" | "::1" | "localhost")
}

#[cfg(any(test, target_os = "windows"))]
fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(target_os = "windows")]
fn default_elevation_checker() -> Result<bool, FirewallError> {
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent()); if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { exit 0 } else { exit 1 }",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| FirewallError::Io("检测管理员权限失败".to_string(), err))?;

    Ok(status.success())
}

#[cfg(not(target_os = "windows"))]
fn default_elevation_checker() -> Result<bool, FirewallError> {
    Ok(true)
}

#[cfg(target_os = "windows")]
fn default_firewall_runner(execution: &FirewallExecution) -> Result<(), FirewallError> {
    let script_path = execution.script_path.display().to_string();
    let direct_args = format!(
        "@('-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-File','{}')",
        escape_powershell_single_quoted(&script_path)
    );

    let status = if execution.requires_elevation {
        let command = format!(
            "$proc = Start-Process -FilePath 'powershell.exe' -Verb RunAs -WindowStyle Hidden -Wait -PassThru -ArgumentList {direct_args}; exit $proc.ExitCode"
        );
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &command,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
    } else {
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script_path,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
    }
    .map_err(|err| FirewallError::Io("执行 Windows 防火墙命令失败".to_string(), err))?;

    if status.success() {
        Ok(())
    } else {
        Err(FirewallError::Command(format!(
            "同步 Windows 防火墙规则失败，退出码: {status}"
        )))
    }
}

#[cfg(not(target_os = "windows"))]
fn default_firewall_runner(_execution: &FirewallExecution) -> Result<(), FirewallError> {
    Ok(())
}
