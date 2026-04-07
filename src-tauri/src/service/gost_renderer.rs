use std::fmt::{Display, Formatter};

use crate::model::rule::{format_socket_addr, Rule};

pub fn render_gost_yaml(rules: &[Rule]) -> Result<String, GostRenderError> {
    let mut yaml = String::from("services:\n");

    for rule in rules {
        if rule.id.trim().is_empty() {
            return Err(GostRenderError::InvalidRule(
                "规则 ID 不能为空，无法渲染 gost.yaml".to_string(),
            ));
        }
        if rule.listen_host.trim().is_empty() {
            return Err(GostRenderError::InvalidRule(format!(
                "规则 {} 的监听地址不能为空，无法渲染 gost.yaml",
                rule.id
            )));
        }
        if rule.target_host.trim().is_empty() {
            return Err(GostRenderError::InvalidRule(format!(
                "规则 {} 的目标地址不能为空，无法渲染 gost.yaml",
                rule.id
            )));
        }

        let service_name = yaml_quote(&format!("rule-{}", rule.id));
        let listen_addr = yaml_quote(&format_socket_addr(&rule.listen_host, rule.listen_port));
        let target_addr = yaml_quote(&format_socket_addr(&rule.target_host, rule.target_port));
        let protocol = rule.protocol.as_str();

        yaml.push_str(&format!(
            "  - name: {service_name}\n    addr: {listen_addr}\n    handler:\n      type: {protocol}\n    listener:\n      type: {protocol}\n    forwarder:\n      nodes:\n        - name: 'target-0'\n          addr: {target_addr}\n"
        ));
    }

    Ok(yaml)
}

fn yaml_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GostRenderError {
    InvalidRule(String),
}

impl Display for GostRenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRule(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for GostRenderError {}
