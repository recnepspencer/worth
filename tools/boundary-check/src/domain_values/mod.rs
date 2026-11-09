//! Per-binding source guard for explicit domain-atom contracts.
mod attributes;
mod claims;
mod closure;
mod macro_origin;
mod modules;
mod type_resolution;
use crate::{
    config::{diagnostic, Code, Diagnostic, Files, CONFIG},
    inputs::Inputs,
};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};

struct Registration {
    source: String,
    binding: String,
    forbidden: Vec<String>,
}
const REGISTRY: &str = "domain-values.toml";

pub fn check(input: &Inputs) -> Vec<Diagnostic> {
    check_registry(input, &format!("{CONFIG}{REGISTRY}"))
}

pub fn check_registry(input: &Inputs, path: &str) -> Vec<Diagnostic> {
    let roots = roots(input);
    match roots.and_then(|roots| check_sources(&input.files, roots, path)) {
        Ok(()) => Vec::new(),
        Err(errors) => errors
            .into_iter()
            .map(|error| diagnostic(Code::DomainValue, path, error))
            .collect(),
    }
}

fn roots(input: &Inputs) -> Result<BTreeMap<String, String>, Vec<String>> {
    let mut roots = BTreeMap::new();
    for graph in input.graphs.values() {
        for package in &graph.packages {
            // --no-deps omits facade packages. Only actual direct dependencies
            // authorize the existing public macro spelling; never type roots.
            if graph.resolve.is_none() {
                for dependency in &package.dependencies {
                    if dependency.name == "worth-query-decl" {
                        roots
                            .entry("worth_query_decl".into())
                            .or_insert_with(|| "external/decl/lib.rs".into());
                    }
                }
            }
            for target in &package.targets {
                if target.kind.iter().any(|kind| kind == "lib")
                    && (input.files.contains_key(&target.src_path)
                        || ["worth_query_decl", "worth_query_declaration"]
                            .contains(&target.name.as_str()))
                {
                    if roots
                        .insert(target.name.clone(), target.src_path.clone())
                        .is_some_and(|previous| previous != target.src_path)
                    {
                        return Err(vec![format!(
                            "ambiguous Cargo library root {}",
                            target.name
                        )]);
                    }
                }
            }
        }
    }
    Ok(roots)
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

fn registrations(files: &Files, path: &str) -> Result<Vec<Registration>, String> {
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
        if entry.len() != 3 {
            return Err("binding requires exactly source, binding, forbidden_types".into());
        }
        let text = |key| {
            entry
                .get(key)
                .and_then(toml::Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("missing/nonliteral {key}"))
        };
        let source = text("source")?;
        let binding = text("binding")?;
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
        });
    }
    Ok(result)
}

fn check_sources(
    files: &Files,
    roots: BTreeMap<String, String>,
    registry: &str,
) -> Result<(), Vec<String>> {
    let registrations = registrations(files, registry).map_err(|error| vec![error])?;
    let sources = modules::Sources { files, roots };
    let mut claims = Vec::new();
    let mut errors = Vec::new();
    for (source, text) in files {
        if source.ends_with(".rs")
            && ["binary", "domain_value", "DOMAIN_VALUE", "domain"]
                .iter()
                .any(|marker| text.contains(marker))
        {
            match claims::discover(source, text) {
                Ok(found) => claims.extend(found),
                Err(error) => errors.push(error),
            }
        }
    }
    let mut used = BTreeSet::new();
    for claim in claims {
        let binding = claim
            .modules
            .iter()
            .cloned()
            .chain([claim.binding.clone()])
            .collect::<Vec<_>>()
            .join("::");
        let matches: Vec<_> = registrations
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.source == claim.source && entry.binding == binding)
            .collect();
        if matches.len() != 1 {
            errors.push(format!(
                "{}::{binding}: explicit domain claim needs exactly one binding registration (unregistered value path {})",
                claim.source, claim.value.to_token_stream()
            ));
            continue;
        }
        let (index, registration) = matches[0];
        if !used.insert(index) {
            errors.push(format!(
                "{}::{binding}: ambiguous duplicate domain claim",
                claim.source
            ));
            continue;
        }
        let result = sources.find_source(&claim.source).and_then(|mut context| {
            context.extend(claim.modules);
            sources.certify_context(&context)?;
            if let Some(origin) = &claim.origin {
                sources.macro_origin(&context, origin, 0)?;
            }
            closure::check(&sources, &context, &claim.value, &registration.forbidden)
        });
        if let Err(error) = result {
            errors.push(format!("{}::{binding}: {error}", claim.source));
        }
    }
    for (index, registration) in registrations.iter().enumerate() {
        if !used.contains(&index) {
            errors.push(format!(
                "{}::{}: stale registration has no actual domain claim",
                registration.source, registration.binding
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests;
