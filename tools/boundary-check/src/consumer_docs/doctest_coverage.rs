//! Genuine rustdoc inclusion in a package selected by the consumer doctest lane.
use super::configuration::Configuration;
use std::path::Path;
use syn::{Expr, Meta};

pub(super) fn validate(
    root: &Path,
    config: &Configuration,
    document: &str,
    name: &str,
) -> Result<(), String> {
    require_ci_lane(root)?;
    let selection = config
        .doctests
        .iter()
        .find(|selection| selection.packages.iter().any(|package| package == name))
        .ok_or_else(|| format!("{name} is not selected by the consumer doctest lane"))?;
    let workspace_manifest = root.join(&selection.manifest);
    let workspace = workspace_manifest.parent().unwrap();
    let manifest_path = workspace.join("crates").join(name).join("Cargo.toml");
    let manifest = crate::source_documents::read_to_string(&manifest_path)
        .map_err(|error| error.to_string())?;
    let manifest: toml::Value = toml::from_str(&manifest).map_err(|error| error.to_string())?;
    if manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        != Some(name)
    {
        return Err(format!("selected package manifest does not declare {name}"));
    }
    if manifest
        .get("lib")
        .and_then(|lib| lib.get("doctest"))
        .and_then(toml::Value::as_bool)
        == Some(false)
    {
        return Err(format!("{name} disables doctests"));
    }
    require_workspace_member(&workspace_manifest, name)?;
    let package_root = manifest_path.parent().unwrap();
    let lib = manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(toml::Value::as_str)
        .unwrap_or("src/lib.rs");
    require_inclusion(root, &package_root.join(lib), document, name)
}

fn require_ci_lane(root: &Path) -> Result<(), String> {
    let ci = crate::source_documents::read_to_string(&root.join(".github/workflows/ci.yml"))
        .map_err(|error| error.to_string())?;
    let workflow: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&ci).map_err(|error| error.to_string())?;
    let runs_lane = workflow["jobs"].as_mapping().is_some_and(|jobs| {
        jobs.values().any(|job| {
            if tolerated(job) || !guaranteed_job(job, &workflow) {
                return false;
            }
            job["steps"].as_sequence().is_some_and(|steps| {
                steps.iter().any(|step| {
                    !conditional_or_tolerated(step)
                        && step["run"].as_str().is_some_and(|command| {
                            command.trim() == "python scripts/ci/query_gate.py doc"
                        })
                })
            })
        })
    });
    if !runs_lane {
        return Err(
            "CI must run the configured doctest lane: python scripts/ci/query_gate.py doc".into(),
        );
    }
    Ok(())
}

fn conditional_or_tolerated(value: &serde_yaml_ng::Value) -> bool {
    !value["if"].is_null() && value["if"].as_bool() != Some(true) || tolerated(value)
}

fn tolerated(value: &serde_yaml_ng::Value) -> bool {
    !value["continue-on-error"].is_null() && value["continue-on-error"].as_bool() != Some(false)
}

fn guaranteed_job(job: &serde_yaml_ng::Value, workflow: &serde_yaml_ng::Value) -> bool {
    if job["if"].is_null() || job["if"].as_bool() == Some(true) {
        return true;
    }
    let Some(guard) = job["if"].as_str() else {
        return false;
    };
    let guard = guard.trim();
    let guard = guard
        .strip_prefix("${{")
        .and_then(|inner| inner.strip_suffix("}}"))
        .unwrap_or(guard)
        .trim();
    if guard != "github.event_name != 'workflow_dispatch' || inputs.scope != 'checkpoint-only'" {
        return false;
    }
    let events = &workflow["on"];
    let scope = &events["workflow_dispatch"]["inputs"]["scope"];
    events.as_mapping().is_some_and(|events| {
        ["push", "pull_request"].iter().all(|event| {
            events
                .get(*event)
                .is_some_and(|declaration| declaration.is_null() || declaration.is_mapping())
        })
    }) && scope["type"].as_str() == Some("choice")
        && scope["default"].as_str() == Some("all")
        && scope["options"].as_sequence().is_some_and(|options| {
            let mut names: Vec<_> = options.iter().filter_map(|value| value.as_str()).collect();
            names.sort_unstable();
            options.len() == 3 && names == ["all", "checkpoint-only", "consumer-docs"]
        })
}

fn require_workspace_member(workspace_manifest: &Path, name: &str) -> Result<(), String> {
    let workspace_text = crate::source_documents::read_to_string(&workspace_manifest)
        .map_err(|error| error.to_string())?;
    let workspace_value: toml::Value =
        toml::from_str(&workspace_text).map_err(|error| error.to_string())?;
    let member = format!("crates/{name}");
    if !workspace_value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .is_some_and(|members| {
            members.iter().any(|value| {
                matches!(value.as_str(), Some("crates/*")) || value.as_str() == Some(&member)
            })
        })
    {
        return Err(format!("{name} is not a member of the selected Cargo root"));
    }
    Ok(())
}

fn require_inclusion(root: &Path, source: &Path, document: &str, name: &str) -> Result<(), String> {
    let source_text =
        crate::source_documents::read_to_string(&source).map_err(|error| error.to_string())?;
    let syntax = crate::source_syntax::parse_named(&source, &source_text)
        .map_err(|error| error.to_string())?;
    if syntax
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr"))
    {
        return Err("doctest inclusion must be in an unconditional library source".into());
    }
    let target = root
        .join(document)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    for attr in &syntax.attrs {
        let Meta::NameValue(meta) = &attr.meta else {
            continue;
        };
        if !meta.path.is_ident("doc") {
            continue;
        }
        let Expr::Macro(expression) = &meta.value else {
            continue;
        };
        if !expression.mac.path.is_ident("include_str") {
            continue;
        }
        let path: syn::LitStr =
            syn::parse2(expression.mac.tokens.clone()).map_err(|error| error.to_string())?;
        if source
            .parent()
            .unwrap()
            .join(path.value())
            .canonicalize()
            .ok()
            .as_ref()
            == Some(&target)
        {
            return Ok(());
        }
    }
    // Plain documentation strings are not evidence that this Markdown file is included.
    Err(format!(
        "{name} library rustdoc does not include {document}"
    ))
}
