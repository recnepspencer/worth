//! Sealed truth is constructed only by its owner, and lifecycle state names
//! its variant.
//!
//! Milestone 3.16.2 requirements 1 and 3 give presented geometry and the
//! presented-frame witness sealed types, and requirement 2 makes lifecycle
//! truth a sum type without defaults. Each covered crate declares its sealed
//! types with their owners, the constructors called outside an owner with the
//! sites that call them, and its lifecycle state.

#[cfg(test)]
mod tests;

mod lifecycle_default;
mod lint_levels;
mod sealed_construction;

use super::production_scope::{production_graphs, production_modules};
use crate::config::TruthTypeDenialConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Check the production source of every configured crate for sealed truth
/// constructed outside its owner, and for `Default` on lifecycle state.
pub(crate) fn enforce_truth_type_denials(
    root: &Path,
    rules: &[TruthTypeDenialConfig],
) -> Vec<Diagnostic> {
    if rules.is_empty() {
        return Vec::new();
    }
    let disallowed_methods = match clippy_disallowed_methods(root) {
        Ok(paths) => paths,
        Err(error) => {
            return vec![Diagnostic::new(
                DiagnosticCode::Bc7005SealedTruthConstruction,
                "clippy.toml",
                format!("Clippy's disallowed methods could not be read: {error}"),
            )]
        }
    };
    let mut diagnostics = Vec::new();
    for rule in rules {
        match production_graphs(root, &rule.crate_root) {
            Ok(graphs) => {
                let modules = production_modules(&graphs);
                diagnostics.extend(sealed_construction::check(
                    &modules,
                    rule,
                    &disallowed_methods,
                ));
                diagnostics.extend(lifecycle_default::check(&modules, rule));
            }
            Err(error) => diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc7005SealedTruthConstruction,
                &rule.crate_root,
                format!("covered crate could not be read for sealed truth: {error}"),
            )),
        }
        match lint_levels::relaxed(root, &rule.crate_root) {
            Ok(relaxed) => diagnostics.extend(relaxed.into_iter().map(|lint| {
                Diagnostic::new(
                    DiagnosticCode::Bc7005SealedTruthConstruction,
                    format!("{}/Cargo.toml", rule.crate_root),
                    format!(
                        "manifest lints allow `{lint}`, which silences Clippy's disallowed methods for the whole crate: {}",
                        rule.guidance
                    ),
                )
            })),
            Err(error) => diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc7005SealedTruthConstruction,
                format!("{}/Cargo.toml", rule.crate_root),
                format!("manifest lint levels could not be read: {error}"),
            )),
        }
    }
    diagnostics
}

/// The method paths the root `clippy.toml` disallows. A mint called as a
/// method is enforced there, because only Clippy resolves its receiver.
fn clippy_disallowed_methods(root: &Path) -> Result<BTreeSet<String>, String> {
    let path = root.join("clippy.toml");
    let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    disallowed_methods_in(&text)
}

fn disallowed_methods_in(text: &str) -> Result<BTreeSet<String>, String> {
    let value: toml::Value = toml::from_str(text).map_err(|error| error.to_string())?;
    let Some(entries) = value.get("disallowed-methods") else {
        return Ok(BTreeSet::new());
    };
    let entries = entries
        .as_array()
        .ok_or("disallowed-methods is not an array")?;
    entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("path").and_then(toml::Value::as_str))
                .map(str::to_owned)
                .ok_or_else(|| format!("disallowed-methods entry {entry} names no path"))
        })
        .collect()
}
