use std::collections::{BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::path::Path;

use crate::model::rule::{Protocol, Rule, RuleSet};
use crate::service::gost_renderer::{render_gost_yaml, GostRenderError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors {
    issues: Vec<String>,
}

impl ValidationErrors {
    pub fn new(mut issues: Vec<String>) -> Self {
        issues.sort();
        issues.dedup();
        Self { issues }
    }

    pub fn issues(&self) -> &[String] {
        &self.issues
    }

    pub fn into_issues(self) -> Vec<String> {
        self.issues
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.issues.join("；"))
    }
}

impl std::error::Error for ValidationErrors {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartValidationOutput {
    pub active_rules: Vec<Rule>,
    pub rendered_config: String,
}

pub fn validate_before_save(rule_set: &RuleSet) -> Result<(), ValidationErrors> {
    let mut issues = Vec::new();
    let mut seen_ids = BTreeSet::new();

    for rule in &rule_set.rules {
        validate_rule_fields(rule, &mut issues);

        if rule.id.trim().is_empty() {
            issues.push("规则 ID 不能为空".to_string());
        } else if !seen_ids.insert(rule.id.clone()) {
            issues.push(format!("规则 ID 重复: {}", rule.id));
        }
    }

    append_listen_conflicts(&rule_set.rules, &mut issues);

    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors::new(issues))
    }
}

pub fn validate_before_start(
    rule_set: &RuleSet,
    active_rule_ids: &BTreeSet<String>,
    sidecar_path: &Path,
) -> Result<StartValidationOutput, ValidationErrors> {
    validate_before_start_with_renderer(rule_set, active_rule_ids, sidecar_path, render_gost_yaml)
}

pub fn validate_before_start_with_renderer<F>(
    rule_set: &RuleSet,
    active_rule_ids: &BTreeSet<String>,
    sidecar_path: &Path,
    render: F,
) -> Result<StartValidationOutput, ValidationErrors>
where
    F: FnOnce(&[Rule]) -> Result<String, GostRenderError>,
{
    let mut issues = Vec::new();
    if active_rule_ids.is_empty() {
        issues.push("启动失败：运行集合为空，请至少选择一条规则".to_string());
    }

    if !sidecar_path.is_file() {
        issues.push(format!(
            "启动失败：未找到 gost sidecar: {}",
            sidecar_path.display()
        ));
    }

    let rules_by_id: HashMap<&str, &Rule> = rule_set
        .rules
        .iter()
        .map(|rule| (rule.id.as_str(), rule))
        .collect();

    let mut active_rules = Vec::new();
    for rule_id in active_rule_ids {
        match rules_by_id.get(rule_id.as_str()) {
            Some(rule) => active_rules.push((*rule).clone()),
            None => issues.push(format!("启动失败：运行集合包含不存在的规则 ID: {rule_id}")),
        }
    }

    for rule in &active_rules {
        validate_rule_fields(rule, &mut issues);
    }
    append_listen_conflicts(&active_rules, &mut issues);

    if !issues.is_empty() {
        return Err(ValidationErrors::new(issues));
    }

    let rendered_config = render(&active_rules).map_err(|err| {
        ValidationErrors::new(vec![format!("启动失败：渲染 gost.yaml 失败: {err}")])
    })?;

    Ok(StartValidationOutput {
        active_rules,
        rendered_config,
    })
}

fn validate_rule_fields(rule: &Rule, issues: &mut Vec<String>) {
    if rule.name.trim().is_empty() {
        issues.push(format!("规则 {} 的名称不能为空", rule.id));
    }

    match rule.protocol {
        Protocol::Tcp | Protocol::Udp => {}
    }

    validate_ip_field(&rule.id, "监听地址", &rule.listen_host, issues);
    validate_ip_field(&rule.id, "目标地址", &rule.target_host, issues);

    if rule.listen_port == 0 {
        issues.push(format!("规则 {} 的监听端口超出范围: 0", rule.id));
    }
    if rule.target_port == 0 {
        issues.push(format!("规则 {} 的目标端口超出范围: 0", rule.id));
    }
}

fn validate_ip_field(rule_id: &str, field_name: &str, host: &str, issues: &mut Vec<String>) {
    if host.trim().is_empty() {
        issues.push(format!("规则 {rule_id} 的{field_name}不能为空"));
        return;
    }

    if host.parse::<IpAddr>().is_err() {
        issues.push(format!(
            "规则 {rule_id} 的{field_name}不是合法 IP 地址: {host}"
        ));
    }
}

fn append_listen_conflicts(rules: &[Rule], issues: &mut Vec<String>) {
    let mut seen: HashMap<(Protocol, String, u16), &str> = HashMap::new();

    for rule in rules {
        let normalized_host = match rule.listen_host.parse::<IpAddr>() {
            Ok(ip) => ip.to_string(),
            Err(_) => continue,
        };

        let key = (rule.protocol, normalized_host, rule.listen_port);
        if let Some(previous_rule_id) = seen.get(&key) {
            issues.push(format!(
                "监听冲突：规则 {} 与规则 {} 不能同时监听 {} {}:{}",
                previous_rule_id,
                rule.id,
                rule.protocol.as_str().to_uppercase(),
                rule.listen_host,
                rule.listen_port
            ));
        } else {
            seen.insert(key, rule.id.as_str());
        }
    }
}
