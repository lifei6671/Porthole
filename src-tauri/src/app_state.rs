use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::fs;

use chrono::Utc;

use crate::model::rule::{Protocol, Rule, RuleSet};
use crate::model::runtime::RuntimeState;
use crate::service::firewall::FirewallManager;
use crate::service::gost_process::{
    GostLaunchRequest, GostProcessError, GostProcessManager, ProcessLogEntry,
};
use crate::service::rule_store::{RuleStore, RuleStoreError};
use crate::service::runtime_events::RuntimeEventEmitter;
use crate::service::validator::{validate_before_save, validate_before_start, ValidationErrors};
use crate::support::paths::AppPaths;

#[derive(Clone)]
pub struct AppState {
    paths: AppPaths,
    rule_store: RuleStore,
    gost_process: GostProcessManager,
    firewall_manager: FirewallManager,
    runtime_events: RuntimeEventEmitter,
    gost_sidecar_path: PathBuf,
    gost_api_probe_url: String,
}

impl AppState {
    pub fn new(
        paths: AppPaths,
        rule_store: RuleStore,
        gost_process: GostProcessManager,
        runtime_events: RuntimeEventEmitter,
        gost_sidecar_path: PathBuf,
        gost_api_probe_url: impl Into<String>,
    ) -> Self {
        let firewall_manager = FirewallManager::new(paths.clone());
        Self {
            paths,
            rule_store,
            gost_process,
            firewall_manager,
            runtime_events,
            gost_sidecar_path,
            gost_api_probe_url: gost_api_probe_url.into(),
        }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn rule_store(&self) -> &RuleStore {
        &self.rule_store
    }

    pub fn gost_process(&self) -> &GostProcessManager {
        &self.gost_process
    }

    pub fn runtime_events(&self) -> &RuntimeEventEmitter {
        &self.runtime_events
    }

    pub fn list_rules(&self) -> Result<Vec<Rule>, AppStateError> {
        Ok(self.rule_store.load()?.rules)
    }

    pub fn create_rule(&self, input: RuleInput) -> Result<Rule, AppStateError> {
        let mut snapshot = self.rule_store.load()?;
        let now = Utc::now();
        let rule = Rule {
            id: generate_rule_id(),
            name: input.name.trim().to_string(),
            enabled: input.enabled,
            protocol: input.protocol,
            listen_host: input.listen_host.trim().to_string(),
            listen_port: input.listen_port,
            target_host: input.target_host.trim().to_string(),
            target_port: input.target_port,
            remark: input.remark.trim().to_string(),
            created_at: now,
            updated_at: now,
        };

        snapshot.rules.push(rule.clone());
        self.save_rules(snapshot)?;
        Ok(rule)
    }

    pub fn update_rule(&self, rule_id: &str, input: RuleInput) -> Result<Rule, AppStateError> {
        let mut snapshot = self.rule_store.load()?;
        let Some(existing) = snapshot.rules.iter_mut().find(|rule| rule.id == rule_id) else {
            return Err(AppStateError::NotFound(format!("未找到规则: {rule_id}")));
        };

        existing.name = input.name.trim().to_string();
        existing.enabled = input.enabled;
        existing.protocol = input.protocol;
        existing.listen_host = input.listen_host.trim().to_string();
        existing.listen_port = input.listen_port;
        existing.target_host = input.target_host.trim().to_string();
        existing.target_port = input.target_port;
        existing.remark = input.remark.trim().to_string();
        existing.updated_at = Utc::now();

        let updated = existing.clone();
        self.save_rules(snapshot)?;
        Ok(updated)
    }

    pub fn delete_rule(&self, rule_id: &str) -> Result<(), AppStateError> {
        let mut snapshot = self.rule_store.load()?;
        let Some(removed_rule) = snapshot.rules.iter().find(|rule| rule.id == rule_id).cloned() else {
            return Err(AppStateError::NotFound(format!("未找到规则: {rule_id}")));
        };
        snapshot.rules.retain(|rule| rule.id != rule_id);

        let active_ids = self.gost_process.runtime_snapshot().active_rule_ids;
        if active_ids.contains(rule_id) {
            let mut next_active_ids = active_ids;
            next_active_ids.remove(rule_id);
            self.apply_runtime_rules(&snapshot, next_active_ids)?;
        }

        self.save_rules(snapshot)?;
        self.sync_removed_firewall_rule(&removed_rule);
        Ok(())
    }

    pub fn start_rule(&self, rule_id: &str) -> Result<RuntimeState, AppStateError> {
        let snapshot = self.rule_store.load()?;
        if !snapshot.rules.iter().any(|rule| rule.id == rule_id) {
            return Err(AppStateError::NotFound(format!("未找到规则: {rule_id}")));
        }

        let mut active_ids = self.gost_process.runtime_snapshot().active_rule_ids;
        active_ids.insert(rule_id.to_string());
        self.apply_runtime_rules(&snapshot, active_ids)
    }

    pub fn stop_rule(&self, rule_id: &str) -> Result<RuntimeState, AppStateError> {
        let snapshot = self.rule_store.load()?;
        let mut active_ids = self.gost_process.runtime_snapshot().active_rule_ids;
        active_ids.remove(rule_id);
        self.apply_runtime_rules(&snapshot, active_ids)
    }

    pub fn start_all_enabled_rules(&self) -> Result<RuntimeState, AppStateError> {
        let snapshot = self.rule_store.load()?;
        let active_ids = snapshot
            .rules
            .iter()
            .filter(|rule| rule.enabled)
            .map(|rule| rule.id.clone())
            .collect::<BTreeSet<_>>();
        self.apply_runtime_rules(&snapshot, active_ids)
    }

    pub fn stop_all_rules(&self) -> Result<RuntimeState, AppStateError> {
        let snapshot = self.rule_store.load()?;
        let runtime = self.gost_process.stop()?;
        self.sync_firewall_rules(&snapshot.rules, &BTreeSet::new());
        self.runtime_events.emit_runtime_changed(runtime.clone());
        Ok(runtime)
    }

    pub fn runtime_snapshot(&self) -> RuntimeState {
        self.gost_process.runtime_snapshot()
    }

    pub fn log_snapshot(&self) -> Vec<ProcessLogEntry> {
        self.gost_process.log_snapshot()
    }

    pub fn clear_logs(&self) {
        self.gost_process.clear_logs();
    }

    fn save_rules(&self, snapshot: RuleSet) -> Result<(), AppStateError> {
        validate_before_save(&snapshot)?;
        self.rule_store.save(&snapshot)?;
        self.runtime_events.emit_rules_changed(snapshot.rules);
        Ok(())
    }

    fn apply_runtime_rules(
        &self,
        snapshot: &RuleSet,
        active_ids: BTreeSet<String>,
    ) -> Result<RuntimeState, AppStateError> {
        if active_ids.is_empty() {
            let runtime = self.gost_process.stop()?;
            self.sync_firewall_rules(&snapshot.rules, &BTreeSet::new());
            self.runtime_events.emit_runtime_changed(runtime.clone());
            return Ok(runtime);
        }

        let gost_sidecar_path = self.prepare_gost_sidecar_path()?;
        let validated = validate_before_start(snapshot, &active_ids, &gost_sidecar_path)?;
        std::fs::write(self.paths.gost_config_file(), &validated.rendered_config)
            .map_err(|err| AppStateError::Io("写入 gost.yaml 失败".to_string(), err))?;

        let request = GostLaunchRequest::new(
            gost_sidecar_path,
            build_gost_args(self.paths.gost_config_file()),
            self.gost_api_probe_url.clone(),
            active_ids.clone(),
        );

        let current_runtime = self.gost_process.runtime_snapshot();
        let runtime = if current_runtime.active_rule_ids.is_empty() {
            self.gost_process.start(request)?
        } else {
            self.gost_process.reload(request)?
        };
        self.sync_firewall_rules(&snapshot.rules, &active_ids);
        self.runtime_events.emit_runtime_changed(runtime.clone());
        Ok(runtime)
    }

    fn sync_firewall_rules(&self, rules: &[Rule], active_ids: &BTreeSet<String>) {
        match self.firewall_manager.sync_rules(rules, active_ids) {
            Ok(messages) => {
                for message in messages {
                    self.gost_process.append_app_info_log(message);
                }
            }
            Err(err) => {
                self.gost_process.append_app_error_log(format!(
                    "同步 Windows 防火墙规则失败，但端口转发仍可继续运行：{err}"
                ));
            }
        }
    }

    fn sync_removed_firewall_rule(&self, rule: &Rule) {
        match self.firewall_manager.remove_rule(rule) {
            Ok(messages) => {
                for message in messages {
                    self.gost_process.append_app_info_log(message);
                }
            }
            Err(err) => {
                self.gost_process.append_app_error_log(format!(
                    "清理 Windows 防火墙规则失败：{err}"
                ));
            }
        }
    }

    fn prepare_gost_sidecar_path(&self) -> Result<PathBuf, AppStateError> {
        #[cfg(target_os = "windows")]
        {
            return self.prepare_windows_gost_sidecar();
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(self.gost_sidecar_path.clone())
        }
    }

    #[cfg(target_os = "windows")]
    fn prepare_windows_gost_sidecar(&self) -> Result<PathBuf, AppStateError> {
        let source = &self.gost_sidecar_path;
        let runtime_path = self.paths.gost_runtime_executable().to_path_buf();

        if !source.exists() {
            return Ok(source.clone());
        }

        if source == &runtime_path {
            return Ok(runtime_path);
        }

        self.paths
            .ensure_sidecar_dir()
            .map_err(|err| AppStateError::Io("创建 sidecar 运行目录失败".to_string(), err))?;

        let should_copy = match (fs::metadata(source), fs::metadata(&runtime_path)) {
            (Ok(source_meta), Ok(runtime_meta)) => {
                source_meta.len() != runtime_meta.len()
                    || source_meta.modified().ok() != runtime_meta.modified().ok()
            }
            (Ok(_), Err(_)) => true,
            (Err(err), _) => {
                return Err(AppStateError::Io(
                    format!("读取 gost sidecar 源文件失败: {}", source.display()),
                    err,
                ));
            }
        };

        if should_copy {
            fs::copy(source, &runtime_path).map_err(|err| {
                AppStateError::Io(
                    format!(
                        "复制 gost sidecar 到本地运行目录失败: {} -> {}",
                        source.display(),
                        runtime_path.display()
                    ),
                    err,
                )
            })?;
        }

        Ok(runtime_path)
    }
}

#[derive(Debug, Clone)]
pub struct RuleInput {
    pub name: String,
    pub enabled: bool,
    pub protocol: Protocol,
    pub listen_host: String,
    pub listen_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub remark: String,
}

#[derive(Debug)]
pub enum AppStateError {
    RuleStore(RuleStoreError),
    Validation(ValidationErrors),
    Process(GostProcessError),
    NotFound(String),
    Io(String, std::io::Error),
}

impl std::fmt::Display for AppStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuleStore(err) => write!(f, "{err}"),
            Self::Validation(err) => write!(f, "{err}"),
            Self::Process(err) => write!(f, "{err}"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Io(message, err) => write!(f, "{message}: {err}"),
        }
    }
}

impl std::error::Error for AppStateError {}

impl From<RuleStoreError> for AppStateError {
    fn from(value: RuleStoreError) -> Self {
        Self::RuleStore(value)
    }
}

impl From<ValidationErrors> for AppStateError {
    fn from(value: ValidationErrors) -> Self {
        Self::Validation(value)
    }
}

impl From<GostProcessError> for AppStateError {
    fn from(value: GostProcessError) -> Self {
        Self::Process(value)
    }
}

fn build_gost_args(config_path: &Path) -> Vec<String> {
    vec!["-C".to_string(), config_path.display().to_string()]
}

fn generate_rule_id() -> String {
    format!(
        "rule-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}
