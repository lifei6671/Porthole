#[path = "../src/model/mod.rs"]
mod model;

#[path = "../src/support/paths.rs"]
pub mod support_paths;

mod support {
    pub use crate::support_paths as paths;
}

#[path = "../src/service/mod.rs"]
mod service;

use std::fs;
use std::sync::Arc;
use std::thread;

use chrono::{TimeZone, Utc};
use model::rule::{Protocol, Rule, RuleSet};
use service::rule_store::RuleStore;
use support::paths::AppPaths;
use tempfile::tempdir;

fn sample_rule(id: &str, name: &str) -> Rule {
    Rule {
        id: id.to_string(),
        name: name.to_string(),
        enabled: true,
        protocol: Protocol::Tcp,
        listen_host: "127.0.0.1".to_string(),
        listen_port: 8080,
        target_host: "10.0.0.10".to_string(),
        target_port: 80,
        remark: "test".to_string(),
        created_at: Utc.with_ymd_and_hms(2026, 4, 7, 10, 11, 12).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 4, 7, 10, 11, 13).unwrap(),
    }
}

#[test]
fn empty_file_is_initialized_with_default_snapshot() {
    let temp_dir = tempdir().expect("create temp dir");
    let paths = AppPaths::new(temp_dir.path());
    fs::write(paths.rules_file(), "").expect("write empty rules file");

    let store = RuleStore::new(paths.clone());
    let snapshot = store.load().expect("load default snapshot");

    assert!(snapshot.rules.is_empty());

    let contents = fs::read_to_string(paths.rules_file()).expect("read rules file");
    assert!(contents.contains("rules = []"));
}

#[test]
fn save_and_reload_round_trip_preserves_rules() {
    let temp_dir = tempdir().expect("create temp dir");
    let paths = AppPaths::new(temp_dir.path());
    let store = RuleStore::new(paths);

    let expected = RuleSet {
        rules: vec![sample_rule("rule-1", "HTTP forward")],
    };

    store.save(&expected).expect("save rules");
    let actual = store.load().expect("reload rules");

    assert_eq!(actual, expected);
}

#[test]
fn timestamps_are_serialized_to_toml_and_can_be_reloaded() {
    let temp_dir = tempdir().expect("create temp dir");
    let paths = AppPaths::new(temp_dir.path());
    let store = RuleStore::new(paths.clone());

    let expected = RuleSet {
        rules: vec![sample_rule("rule-2", "TCP forward")],
    };

    store.save(&expected).expect("save rules");

    let contents = fs::read_to_string(paths.rules_file()).expect("read rules file");
    assert!(contents.contains("2026-04-07T10:11:12Z"));
    assert!(contents.contains("2026-04-07T10:11:13Z"));

    let actual = store.load().expect("reload rules");
    assert_eq!(actual, expected);
}

#[test]
fn concurrent_saves_do_not_corrupt_rules_file() {
    let temp_dir = tempdir().expect("create temp dir");
    let paths = AppPaths::new(temp_dir.path());
    let store = Arc::new(RuleStore::new(paths.clone()));

    let mut handles = Vec::new();
    for index in 0..4 {
        let store = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            let snapshot = RuleSet {
                rules: vec![sample_rule(
                    &format!("rule-{index}"),
                    &format!("Rule {index}"),
                )],
            };
            store.save(&snapshot).expect("save concurrent snapshot");
        }));
    }

    for handle in handles {
        handle.join().expect("join writer thread");
    }

    let contents = fs::read_to_string(paths.rules_file()).expect("read rules file");
    let parsed: RuleSet = toml::from_str(&contents).expect("parse final rules file");

    assert_eq!(parsed.rules.len(), 1);
    assert!(parsed.rules[0].id.starts_with("rule-"));
}
