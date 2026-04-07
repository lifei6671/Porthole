#[path = "../src/model/mod.rs"]
mod model;

#[path = "../src/support/paths.rs"]
pub mod support_paths;

mod support {
    pub use crate::support_paths as paths;
}

#[path = "../src/service/firewall.rs"]
mod firewall;

use std::collections::BTreeSet;

use chrono::Utc;
use firewall::{build_sync_plan, firewall_rule_name, is_loopback_host, prune_sync_plan};
use model::rule::{Protocol, Rule};

fn sample_rule(id: &str, protocol: Protocol, listen_host: &str, listen_port: u16) -> Rule {
    Rule {
        id: id.to_string(),
        name: format!("Rule {id}"),
        enabled: true,
        protocol,
        listen_host: listen_host.to_string(),
        listen_port,
        target_host: "127.0.0.1".to_string(),
        target_port: 80,
        remark: String::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn build_sync_plan_only_includes_non_loopback_rules() {
    let active_ids = BTreeSet::from(["rule-a".to_string()]);
    let plan = build_sync_plan(
        &[
            sample_rule("rule-a", Protocol::Tcp, "0.0.0.0", 8080),
            sample_rule("rule-b", Protocol::Udp, "127.0.0.1", 5353),
        ],
        &active_ids,
    );

    assert_eq!(plan.add_rules.len(), 1);
    assert_eq!(plan.remove_rule_names.len(), 0);
    assert_eq!(plan.add_rules[0].display_name, "Porthole-rule-a-tcp-8080");
    assert_eq!(plan.add_rules[0].local_port, 8080);
}

#[test]
fn build_sync_plan_removes_inactive_exposed_rules() {
    let rules = vec![
        sample_rule("rule-a", Protocol::Tcp, "0.0.0.0", 8080),
        sample_rule("rule-b", Protocol::Udp, "::", 5353),
    ];
    let active_ids = BTreeSet::from(["rule-b".to_string()]);
    let plan = build_sync_plan(&rules, &active_ids);

    assert_eq!(plan.add_rules.len(), 1);
    assert_eq!(plan.remove_rule_names, vec!["Porthole-rule-a-tcp-8080"]);
    assert_eq!(plan.add_rules[0].display_name, "Porthole-rule-b-udp-5353");
}

#[test]
fn powershell_script_contains_add_and_remove_commands() {
    let rules = vec![
        sample_rule("rule-a", Protocol::Tcp, "0.0.0.0", 8080),
        sample_rule("rule-b", Protocol::Udp, "::", 5353),
    ];
    let active_ids = BTreeSet::from(["rule-a".to_string()]);
    let plan = build_sync_plan(&rules, &active_ids);
    let script = plan.to_powershell_script();

    assert!(script.contains("New-NetFirewallRule"));
    assert!(script.contains("Porthole-rule-a-tcp-8080"));
    assert!(script.contains("Remove-NetFirewallRule"));
    assert!(script.contains("Porthole-rule-b-udp-5353"));
    assert!(script.contains("Protocol TCP"));
    assert!(script.contains("LocalPort 8080"));
}

#[test]
fn helper_functions_match_expected_rule_names_and_hosts() {
    let rule = sample_rule("rule-x", Protocol::Udp, "localhost", 9999);
    assert_eq!(firewall_rule_name(&rule), "Porthole-rule-x-udp-9999");
    assert!(is_loopback_host("127.0.0.1"));
    assert!(is_loopback_host("::1"));
    assert!(is_loopback_host("localhost"));
    assert!(!is_loopback_host("0.0.0.0"));
    assert!(!is_loopback_host("::"));
}

#[test]
fn prune_sync_plan_only_keeps_real_changes() {
    let rules = vec![
        sample_rule("rule-a", Protocol::Tcp, "0.0.0.0", 8080),
        sample_rule("rule-b", Protocol::Udp, "::", 5353),
    ];
    let active_ids = BTreeSet::from(["rule-a".to_string()]);
    let plan = build_sync_plan(&rules, &active_ids);
    let existing = BTreeSet::from([
        "Porthole-rule-a-tcp-8080".to_string(),
        "Porthole-rule-b-udp-5353".to_string(),
    ]);

    let pruned = prune_sync_plan(plan, &existing);

    assert!(pruned.add_rules.is_empty());
    assert_eq!(pruned.remove_rule_names, vec!["Porthole-rule-b-udp-5353"]);
}
