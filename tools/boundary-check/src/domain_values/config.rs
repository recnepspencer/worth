//! Reviewed binding policy; source declarations remain the value truth.
use crate::config::Files;
use std::collections::BTreeSet;

pub(super) enum Posture {
    Domain,
    Exception,
    Diagnostic,
}
pub(super) struct Registration {
    pub source: String,
    pub binding: String,
    pub forbidden: Vec<String>,
    pub posture: Posture,
}

fn registry_table(files: &Files, path: &str) -> Result<toml::Table, String> {
    let text = files
        .get(path)
        .ok_or_else(|| format!("required domain registry missing: {path}"))?;
    let value: toml::Value = toml::from_str(text).map_err(|error| error.to_string())?;
    let table = value.as_table().ok_or("domain registry must be a table")?;
    if table
        .keys()
        .any(|key| !["binding", "workspace_manifests"].contains(&key.as_str()))
    {
        return Err("domain registry accepts only binding entries and workspace_manifests".into());
    }
    Ok(table.clone())
}

/// Actual Cargo manifests augment the shared metadata inventory, never type roots.
pub fn workspace_manifests(files: &Files, path: &str) -> Result<Vec<String>, String> {
    let table = registry_table(files, path)?;
    let Some(value) = table.get("workspace_manifests") else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or("workspace_manifests must be an array")?;
    array
        .iter()
        .map(|value| {
            let path = value
                .as_str()
                .ok_or("workspace manifest must be a literal path")?;
            if !path.ends_with("Cargo.toml") || !files.contains_key(path) {
                return Err(format!(
                    "domain workspace manifest is outside Git inputs: {path}"
                ));
            }
            Ok(path.to_owned())
        })
        .collect()
}

pub(super) fn registrations(files: &Files, path: &str) -> Result<Vec<Registration>, String> {
    let table = registry_table(files, path)?;
    workspace_manifests(files, path)?;
    let entries = table
        .get("binding")
        .and_then(toml::Value::as_array)
        .ok_or("domain registry needs binding array")?;
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in entries {
        let entry = entry.as_table().ok_or("binding entry must be a table")?;
        let text = |key| {
            entry
                .get(key)
                .and_then(toml::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("missing/nonliteral {key}"))
        };
        let source = text("source")?;
        let binding = text("binding")?;
        let posture = match text("posture")
            .map_err(|error| format!("{source}::{binding}: {error}"))?
            .as_str()
        {
            "domain" => Posture::Domain,
            "exception" => Posture::Exception,
            "diagnostic" => Posture::Diagnostic,
            _ => {
                return Err(format!(
                    "{source}::{binding}: posture must be domain, exception or diagnostic"
                ))
            }
        };
        text("reason").map_err(|error| format!("{source}::{binding}: {error}"))?;
        if entry.len() != 5 {
            return Err(
                "binding requires exactly source, binding, forbidden_types, posture, reason".into(),
            );
        }
        let forbidden = entry
            .get("forbidden_types")
            .and_then(toml::Value::as_array)
            .ok_or("forbidden_types must be explicit")?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .ok_or("invalid forbidden type")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !seen.insert((source.clone(), binding.clone())) {
            return Err("duplicate domain binding registration".into());
        }
        result.push(Registration {
            source,
            binding,
            forbidden,
            posture,
        });
    }
    Ok(result)
}
