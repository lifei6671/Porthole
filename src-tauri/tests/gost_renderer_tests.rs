#[path = "../src/model/mod.rs"]
mod model;

#[path = "../src/support/paths.rs"]
pub mod support_paths;

mod support {
    pub use crate::support_paths as paths;
}

#[path = "../src/service/mod.rs"]
mod service;

use chrono::{TimeZone, Utc};
use model::rule::{format_socket_addr, Protocol, Rule};
use service::gost_renderer::render_gost_yaml;

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
fn format_socket_addr_wraps_ipv6_only_when_needed() {
    assert_eq!(format_socket_addr("127.0.0.1", 8080), "127.0.0.1:8080");
    assert_eq!(format_socket_addr("::1", 8080), "[::1]:8080");
}

#[test]
fn render_supports_ipv4_to_ipv4_tcp_rule() {
    let yaml = render_gost_yaml(&[sample_rule(
        "rule-v4-v4",
        Protocol::Tcp,
        "127.0.0.1",
        8080,
        "10.0.0.8",
        80,
    )])
    .expect("render yaml");

    assert!(yaml.contains("type: tcp"));
    assert!(yaml.contains("addr: '127.0.0.1:8080'"));
    assert!(yaml.contains("addr: '10.0.0.8:80'"));
}

#[test]
fn render_supports_ipv6_to_ipv6_udp_rule_without_metadata() {
    let yaml = render_gost_yaml(&[sample_rule(
        "rule-v6-v6",
        Protocol::Udp,
        "::1",
        5353,
        "2001:db8::10",
        5353,
    )])
    .expect("render yaml");

    assert!(yaml.contains("type: udp"));
    assert!(yaml.contains("addr: '[::1]:5353'"));
    assert!(yaml.contains("addr: '[2001:db8::10]:5353'"));
    assert!(!yaml.contains("metadata:"));
    assert!(!yaml.contains("keepAlive"));
    assert!(!yaml.contains("ttl"));
}

#[test]
fn render_supports_ipv4_to_ipv6_rule() {
    let yaml = render_gost_yaml(&[sample_rule(
        "rule-v4-v6",
        Protocol::Tcp,
        "0.0.0.0",
        7000,
        "::1",
        7001,
    )])
    .expect("render yaml");

    assert!(yaml.contains("addr: '0.0.0.0:7000'"));
    assert!(yaml.contains("addr: '[::1]:7001'"));
}

#[test]
fn render_supports_ipv6_to_ipv4_rule() {
    let yaml = render_gost_yaml(&[sample_rule(
        "rule-v6-v4",
        Protocol::Udp,
        "::",
        9000,
        "192.168.0.10",
        9001,
    )])
    .expect("render yaml");

    assert!(yaml.contains("addr: '[::]:9000'"));
    assert!(yaml.contains("addr: '192.168.0.10:9001'"));
}
