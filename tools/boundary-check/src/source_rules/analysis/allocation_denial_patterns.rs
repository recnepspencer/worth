//! Allocation request/physical classification belongs to Execution alone.
mod alias_declarations;
mod macro_paths;
mod raw_kind_types;
mod source;
#[cfg(test)]
mod tests;

use super::crate_modules::ModuleGraph;
use super::production_crate_roots::crate_roots;
use super::production_scope::{production_graphs, production_modules};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(super) fn validate(root: &Path) -> Result<Vec<Diagnostic>, String> {
    let mut roots = crate_roots(root)?;
    discover_nested(root, &root.join("tools"), &mut roots)?;
    // Consumer fixtures are independent Cargo roots below the member crates.
    for crate_root in roots.clone() {
        discover_nested(root, &root.join(crate_root).join("fixtures"), &mut roots)?;
    }
    roots.sort();
    roots.dedup();
    let mut diagnostics = Vec::new();
    for crate_root in roots {
        if crate_root == "crates/worth-execution" || !mentions_raw_kind(&root.join(&crate_root))? {
            continue;
        }
        diagnostics.extend(check_graphs(
            &crate_root,
            &production_graphs(root, &crate_root)?,
        ));
    }
    Ok(diagnostics)
}

fn discover_nested(root: &Path, directory: &Path, roots: &mut Vec<String>) -> Result<(), String> {
    if !directory.is_dir() {
        return Ok(());
    }
    let manifest = directory.join("Cargo.toml");
    if manifest.is_file()
        && crate::source_documents::read_to_string(&manifest)
            .map_err(|e| e.to_string())?
            .parse::<toml::Value>()
            .map_err(|e| e.to_string())?
            .get("package")
            .is_some()
    {
        roots.push(
            directory
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/"),
        );
    }
    for entry in
        fs::read_dir(directory).map_err(|e| format!("read {}: {e}", directory.display()))?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir()
            && !path.file_name().is_some_and(|name| {
                matches!(name.to_str(), Some("target" | ".git" | "node_modules"))
            })
        {
            discover_nested(root, &path, roots)?;
        }
    }
    Ok(())
}

/// Lexical absence is sufficient to skip parsing an unrelated crate. Presence
/// is only a candidate: production module and pattern syntax decide legality.
fn mentions_raw_kind(directory: &Path) -> Result<bool, String> {
    for entry in
        fs::read_dir(directory).map_err(|e| format!("read {}: {e}", directory.display()))?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            if !path.file_name().is_some_and(|name| {
                matches!(name.to_str(), Some("target" | ".git" | "node_modules"))
            }) && mentions_raw_kind(&path)?
            {
                return Ok(true);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs")
            && crate::source_documents::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?
                .contains("ExecutionAllocationDenialKind")
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn check_graphs(crate_root: &str, graphs: &[ModuleGraph]) -> Vec<Diagnostic> {
    if crate_root == "crates/worth-execution" {
        return Vec::new();
    }
    let mut found = BTreeSet::new();
    let aliases = raw_kind_types::Aliases::from_graphs(graphs);
    for module in production_modules(graphs) {
        for (item, line) in source::findings(module.items, &aliases, module.relative_source) {
            found.insert((module.relative_source, item, line));
        }
    }
    found.into_iter().map(|(path, item, line)| Diagnostic::new(
        DiagnosticCode::Bc7010AllocationDenialPattern,
        format!("{crate_root}/{path}:{line}"),
        format!("raw allocation kind access or alias in `{item}`; call denial.classify() and match the classified value; raw variants, aliases and trait impls belong to worth-execution"),
    )).collect()
}
