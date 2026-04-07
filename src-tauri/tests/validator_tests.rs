#[path = "../src/model/mod.rs"]
mod model;

#[path = "../src/support/paths.rs"]
pub mod support_paths;

mod support {
    pub use crate::support_paths as paths;
}

#[path = "../src/service/mod.rs"]
mod service;

use std::collections::BTreeSet;
use std::fs;

use chrono::{TimeZone, Utc};
use model::rule::{Protocol, Rule, RuleSet};
use service::validator::{
    validate_before_save, validate_before_start, validate_before_start_with_renderer,
};
use tempfile::tempdir;

fn sample_rule(
    id: &str,
    protocol: Protocol,
    listen_host: &str,
    listen_port: u16,
    target_host: &str,
    target_port: u16,
) -> Rule {
    Rule {
        id: id.to_string(),
        name: format!("Rule {id}"),
        enabled: true,
        protocol,
        listen_host: listen_host.to_string(),
        listen_port,
        target_host: target_host.to_string(),
        target_port,
        remark: String::new(),
        created_at: Utc.with_ymd_and_hms(2026, 4, 7, 12, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 4, 7, 12, 0, 1).unwrap(),
    }
}

#[test]
fn save_validation_rejects_duplicate_ids_and_conflicting_listeners() {
    let rules = RuleSet {
        rules: vec![
            sample_rule("dup", Protocol::Tcp, "127.0.0.1", 8080, "10.0.0.1", 80),
            sample_rule("dup", Protocol::Tcp, "127.0.0.1", 8080, "10.0.0.2", 81),
        ],
    };

    let err = validate_before_save(&rules).expect_err("validation should fail");
    let issues = err.into_issues();

    assert!(issues.iter().any(|item| item.contains("规则 ID 重复: dup")));
    assert!(issues
        .iter()
        .any(|item| item.contains("监听冲突：规则 dup 与规则 dup")));
}

#[test]
fn save_validation_rejects_invalid_ip_and_zero_port() {
    let rules = RuleSet {
        rules: vec![sample_rule(
            "bad-rule",
            Protocol::Udp,
            "not-an-ip",
            0,
            "127.0.0.1",
            0,
        )],
    };

    let err = validate_before_save(&rules).expect_err("validation should fail");
    let issues = err.into_issues();

    assert!(issues
        .iter()
        .any(|item| item.contains("监听地址不是合法 IP 地址")));
    assert!(issues
        .iter()
        .any(|item| item.contains("监听端口超出范围: 0")));
    assert!(issues
        .iter()
        .any(|item| item.contains("目标端口超出范围: 0")));
}

#[test]
fn start_validation_rejects_empty_runtime_set_and_missing_sidecar() {
    let rules = RuleSet {
        rules: vec![sample_rule(
            "rule-1",
            Protocol::Tcp,
            "127.0.0.1",
            8080,
            "10.0.0.1",
            80,
        )],
    };

    let err = validate_before_start(
        &rules,
        &BTreeSet::new(),
        std::path::Path::new("missing.exe"),
    )
    .expect_err("start validation should fail");
    let issues = err.into_issues();

    assert!(issues.iter().any(|item| item.contains("运行集合为空")));
    assert!(issues
        .iter()
        .any(|item| item.contains("未找到 gost sidecar")));
}

#[test]
fn start_validation_rejects_render_failures() {
    let rules = RuleSet {
        rules: vec![sample_rule(
            "rule-1",
            Protocol::Tcp,
            "127.0.0.1",
            8080,
            "10.0.0.1",
            80,
        )],
    };
    let active_rule_ids = BTreeSet::from(["rule-1".to_string()]);
    let temp_dir = tempdir().expect("create temp dir");
    let sidecar_path = temp_dir.path().join("gost.exe");
    fs::write(&sidecar_path, b"fake").expect("write fake sidecar");

    let err = validate_before_start_with_renderer(&rules, &active_rule_ids, &sidecar_path, |_| {
        Err(service::gost_renderer::GostRenderError::InvalidRule(
            "模拟渲染失败".to_string(),
        ))
    })
    .expect_err("start validation should fail");

    assert!(err
        .into_issues()
        .iter()
        .any(|item| item.contains("渲染 gost.yaml 失败: 模拟渲染失败")));
}

#[test]
fn start_validation_returns_rendered_config_for_valid_runtime_set() {
    let rules = RuleSet {
        rules: vec![
            sample_rule("rule-1", Protocol::Tcp, "127.0.0.1", 8080, "10.0.0.1", 80),
            sample_rule("rule-2", Protocol::Udp, "::1", 5353, "127.0.0.1", 5353),
        ],
    };
    let active_rule_ids = BTreeSet::from(["rule-2".to_string()]);
    let temp_dir = tempdir().expect("create temp dir");
    let sidecar_path = temp_dir.path().join("gost.exe");
    fs::write(&sidecar_path, b"fake").expect("write fake sidecar");

    let output =
        validate_before_start(&rules, &active_rule_ids, &sidecar_path).expect("validation pass");

    assert_eq!(output.active_rules.len(), 1);
    assert_eq!(output.active_rules[0].id, "rule-2");
    assert!(output.rendered_config.contains("type: udp"));
    assert!(output.rendered_config.contains("addr: '[::1]:5353'"));
}
