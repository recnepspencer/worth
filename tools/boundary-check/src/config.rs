//! Configuration and the stable diagnostic wire contract.
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use std::collections::BTreeMap;

pub const CONFIG: &str = "tools/boundary-check/config/";
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Diagnostic {
    pub code: &'static str,
    pub subject: String,
    pub message: String,
    pub legal_home: String,
}
pub fn diagnostic(code: &'static str, subject: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        subject: subject.into(),
        message: message.into(),
        legal_home: format!("{CONFIG}road1.toml"),
    }
}
pub type Files = BTreeMap<String, String>;
pub fn under(path: &str, directory: &str) -> bool {
    path == directory
        || path
            .strip_prefix(directory.trim_end_matches('/'))
            .is_some_and(|s| s.starts_with('/'))
}
pub fn governed(path: &str) -> bool {
    ![
        "workspaces/worth-store",
        "workspaces/worth-ui",
        "crates/worth-proof",
    ]
    .iter()
    .any(|p| under(path, p))
}
pub fn table<'a>(value: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|v| v.iter().collect())
        .unwrap_or_default()
}
pub fn strings(value: &toml::Value, key: &str) -> Vec<String> {
    table(value, key)
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}
pub fn string<'a>(value: &'a toml::Value, key: &str) -> &'a str {
    value.get(key).and_then(toml::Value::as_str).unwrap_or("")
}
pub fn pairs(text: &str) -> Vec<(&str, &str)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split_once(' '))
        .collect()
}
/// Admit kept policy fields before running graph processes or rules.
pub fn validate_policy(config: &toml::Value) -> Result<(), String> {
    for (section, text_fields, list_fields) in [
        (
            "dependency_denials",
            &["workspace_manifest", "guidance"][..],
            &["sources", "forbidden_targets"][..],
        ),
        (
            "source_dependency_allowlists",
            &["workspace_manifest", "guidance"][..],
            &["sources", "allowed_targets"][..],
        ),
        (
            "dependency_target_allowlists",
            &["workspace_manifest", "target", "guidance"][..],
            &["governed_source_prefixes", "allowed_sources"][..],
        ),
        (
            "source_identifier_denials",
            &["root", "guidance"][..],
            &["forbidden_identifiers"][..],
        ),
    ] {
        let Some(value) = config.get(section) else {
            continue;
        };
        let rules = value
            .as_array()
            .ok_or_else(|| format!("{section} must be a table array"))?;
        for rule in rules {
            if let Some(contracts) = rule.get("dependency_contracts") {
                let contracts = contracts
                    .as_array()
                    .ok_or("dependency_contracts must be a table array")?;
                for contract in contracts {
                    if ["target", "version_requirement"]
                        .iter()
                        .any(|k| contract.get(*k).and_then(toml::Value::as_str).is_none())
                        || contract
                            .get("uses_default_features")
                            .and_then(toml::Value::as_bool)
                            .is_none()
                        || contract
                            .get("features")
                            .and_then(toml::Value::as_array)
                            .is_none_or(|a| a.iter().any(|v| v.as_str().is_none()))
                    {
                        return Err("invalid dependency contract".into());
                    }
                }
            }
            for key in text_fields {
                if rule.get(*key).and_then(toml::Value::as_str).is_none() {
                    return Err(format!("{section}.{key} must be a string"));
                }
            }
            for key in list_fields {
                if rule
                    .get(*key)
                    .and_then(toml::Value::as_array)
                    .is_none_or(|a| a.iter().any(|v| v.as_str().is_none()))
                {
                    return Err(format!("{section}.{key} must be a string array"));
                }
            }
            for key in [
                "source_prefixes",
                "forbidden_target_prefixes",
                "exclude_paths",
                "forbidden_identifier_fragments",
            ] {
                if rule.get(key).is_some_and(|v| {
                    v.as_array()
                        .is_none_or(|a| a.iter().any(|v| v.as_str().is_none()))
                }) {
                    return Err(format!("{section}.{key} must be a string array"));
                }
            }
        }
    }
    Ok(())
}
pub fn validate_auxiliary(name: &str, value: &toml::Value) -> Result<(), String> {
    let fields: &[(&str, &[&str])] = match name {
        "runtime-roots.toml" => &[("roots", &["type"])],
        "consumer-docs.toml" => &[
            (
                "compiled_sources",
                &["name", "crate_root", "target", "source"],
            ),
            ("facts", &["id", "manifest", "item", "document"]),
        ],
        "court-frontier-splits.toml" => &[("splits", &["court", "area"])],
        _ => &[],
    };
    if name == "runtime-roots.toml" && value.get("roots").is_none() {
        return Err("runtime root registry missing".into());
    }
    if name == "consumer-docs.toml"
        && value
            .get("documents")
            .and_then(toml::Value::as_array)
            .is_none_or(|a| a.iter().any(|v| v.as_str().is_none()))
    {
        return Err("consumer documents must be a string array".into());
    }
    for (key, strings) in fields {
        if let Some(rows) = value.get(*key) {
            let rows = rows
                .as_array()
                .ok_or_else(|| format!("{name}.{key} must be a table array"))?;
            if rows.iter().any(|r| {
                strings
                    .iter()
                    .any(|k| r.get(*k).and_then(toml::Value::as_str).is_none())
            }) {
                return Err(format!("{name}.{key} has invalid fields"));
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_policy_arrays_are_refused_before_rules_run() {
        assert!(validate_policy(&toml::from_str("[[dependency_denials]]\nworkspace_manifest='Cargo.toml'\nsources=[]\nforbidden_targets=[]\nguidance='closed'").unwrap()).is_ok());
        assert!(
            validate_policy(&toml::from_str("dependency_denials='disabled'").unwrap()).is_err()
        );
        assert!(validate_auxiliary(
            "consumer-docs.toml",
            &toml::from_str("documents='disabled'").unwrap()
        )
        .is_err());
    }
}
#[derive(Default)]
pub struct Metadata {
    pub packages: Vec<Package>,
    pub workspace_members: Vec<String>,
    pub resolve: Option<Resolve>,
}
#[derive(Default)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub manifest_path: String,
    pub dependencies: Vec<Dependency>,
    pub targets: Vec<Target>,
}
#[derive(Default)]
pub struct Dependency {
    pub name: String,
    pub kind: Option<String>,
    pub req: String,
    pub uses_default_features: bool,
    pub features: Vec<String>,
}
#[derive(Default)]
pub struct Target {
    pub name: String,
    pub kind: Vec<String>,
    pub src_path: String,
}
#[derive(Default)]
pub struct Resolve {
    pub nodes: Vec<Node>,
}
#[derive(Default)]
pub struct Node {
    pub id: String,
    pub deps: Vec<Edge>,
}
#[derive(Default)]
pub struct Edge {
    pub pkg: String,
    pub dep_kinds: Vec<Kind>,
}
#[derive(Default)]
pub struct Kind {
    pub kind: Option<String>,
}

impl Serialize for Diagnostic {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Diagnostic", 4)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("subject", &self.subject)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("legal_home", &self.legal_home)?;
        state.end()
    }
}
