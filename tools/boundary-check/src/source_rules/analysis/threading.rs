//! Production threading and parallel-computation sites, with an exact shrinking
//! inventory for the lanes being moved into worth-execution.

mod source;

use super::production_scope::{production_graphs, production_modules};
use crate::config::ThreadingSiteConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Site {
    source: String,
    item: String,
    kind: String,
}

pub(crate) fn enforce_threading_boundary(
    root: &Path,
    declared: &[ThreadingSiteConfig],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let roots = match crate_roots(root) {
        Ok(roots) => roots,
        Err(error) => return vec![diagnostic("crates/", error)],
    };
    let mut observed = BTreeMap::<Site, usize>::new();
    for crate_root in roots {
        if crate_root.ends_with("crates/worth-execution") {
            continue;
        }
        if let Err(error) = observe_crate(root, &crate_root, &mut observed) {
            diagnostics.push(diagnostic(&crate_root, error));
        }
    }
    let mut declared_keys = BTreeSet::new();
    for entry in declared {
        let key = Site {
            source: entry.source.clone(),
            item: entry.item.clone(),
            kind: entry.kind.clone(),
        };
        if !declared_keys.insert(key.clone()) {
            diagnostics.push(diagnostic(
                &entry.source,
                format!(
                    "duplicate threading declaration for {} / {}",
                    entry.item, entry.kind
                ),
            ));
            continue;
        }
        if let Some(error) = invalid_declaration(entry) {
            diagnostics.push(diagnostic(&entry.source, error));
        }
        let found = observed.remove(&key).unwrap_or(0);
        if found != entry.count {
            diagnostics.push(diagnostic(&entry.source, format!("threading declaration `{}` / `{}` expects {} production site(s), found {found}; remove a stale entry or account for a new site", entry.item, entry.kind, entry.count)));
        }
    }
    diagnostics.extend(observed.into_iter().map(|(site, count)| {
        diagnostic(&site.source, format!("undeclared production threading site `{}` / `{}` ({count} occurrence(s)); move parallel work to worth-execution or declare a non-compute thread", site.item, site.kind))
    }));
    diagnostics
}

fn invalid_declaration(entry: &ThreadingSiteConfig) -> Option<String> {
    if entry.source.is_empty()
        || !entry.source.ends_with(".rs") && !entry.source.ends_with("/Cargo.toml")
        || entry.item.is_empty() && entry.kind != "rayon-import" && entry.kind != "rayon-dependency"
    {
        return Some("threading declaration needs an exact production source and item".into());
    }
    if entry.count == 0 || entry.reason.trim().is_empty() {
        return Some("threading declaration needs a positive exact count and reason".into());
    }
    let is_parallel = matches!(
        entry.kind.as_str(),
        "rayon-import"
            | "rayon-path"
            | "rayon-call"
            | "rayon-dependency"
            | "pool-dependency"
            | "thread-pool"
    ) || matches!(
        entry.kind.as_str(),
        "thread-spawn" | "thread-scope" | "thread-builder"
    ) && entry.category == "legacy-parallel";
    if entry.category == "legacy-parallel" {
        if !is_parallel || !matches!(entry.retire_phase, Some(3 | 4 | 7)) {
            return Some(
                "legacy parallel site needs a valid kind and retirement phase 3, 4 or 7".into(),
            );
        }
    } else if !matches!(
        entry.category.as_str(),
        "owner-thread"
            | "io-durability-worker"
            | "event-loop-watcher"
            | "stream-reader"
            | "async-runtime"
            | "certification-harness"
    ) || entry.retire_phase.is_some()
        || is_parallel
    {
        return Some("non-compute threading site needs a declared category without a retirement phase; Rayon belongs to the legacy parallel ratchet".into());
    }
    None
}

fn observe_crate(
    root: &Path,
    crate_root: &str,
    found: &mut BTreeMap<Site, usize>,
) -> Result<(), String> {
    let manifest = root.join(crate_root).join("Cargo.toml");
    for kind in parallel_dependencies(&manifest)? {
        record(
            found,
            format!("{crate_root}/Cargo.toml"),
            "dependencies".into(),
            kind.into(),
        );
    }
    let graphs = production_graphs(root, crate_root)?;
    for module in production_modules(&graphs) {
        for source in module.sources() {
            let mut visitor = source::ThreadingCalls::for_items(module.items);
            for (item, kind) in visitor.take_sites() {
                record(found, format!("{crate_root}/{source}"), item, kind);
            }
        }
    }
    Ok(())
}

fn record(found: &mut BTreeMap<Site, usize>, source: String, item: String, kind: String) {
    *found.entry(Site { source, item, kind }).or_default() += 1;
}

fn parallel_dependencies(manifest: &Path) -> Result<Vec<&'static str>, String> {
    let text = fs::read_to_string(manifest)
        .map_err(|error| format!("read {}: {error}", manifest.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|error| format!("parse {}: {error}", manifest.display()))?;
    fn collect(value: &toml::Value, found: &mut Vec<&'static str>) {
        let Some(table) = value.as_table() else {
            return;
        };
        for (key, value) in table {
            if matches!(key.as_str(), "dependencies" | "build-dependencies") {
                if let Some(deps) = value.as_table() {
                    for (name, declaration) in deps {
                        let package = declaration
                            .get("package")
                            .and_then(toml::Value::as_str)
                            .unwrap_or(name);
                        match package {
                            "rayon" => found.push("rayon-dependency"),
                            "threadpool" | "scoped_threadpool" => found.push("pool-dependency"),
                            _ => {}
                        }
                    }
                }
            } else if key == "target" {
                if let Some(targets) = value.as_table() {
                    for target in targets.values() {
                        collect(target, found);
                    }
                }
            }
        }
    }
    let mut found = Vec::new();
    collect(&value, &mut found);
    Ok(found)
}

fn crate_roots(root: &Path) -> Result<Vec<String>, String> {
    let mut lanes = vec![PathBuf::from("crates")];
    let workspace_root = root.join("workspaces");
    if workspace_root.is_dir() {
        for entry in
            fs::read_dir(&workspace_root).map_err(|error| format!("read workspaces: {error}"))?
        {
            let entry = entry.map_err(|error| format!("read workspace entry: {error}"))?;
            if entry.path().is_dir() {
                let relative = PathBuf::from("workspaces").join(entry.file_name());
                lanes.push(relative.join("crates"));
                lanes.push(relative.join("apps"));
                lanes.push(relative.join("tools"));
            }
        }
    }
    let mut roots = Vec::new();
    for lane in lanes {
        let absolute = root.join(&lane);
        if !absolute.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&absolute)
            .map_err(|error| format!("read {}: {error}", absolute.display()))?
        {
            let entry = entry.map_err(|error| format!("read crate entry: {error}"))?;
            if entry.path().join("Cargo.toml").is_file() {
                roots.push(
                    lane.join(entry.file_name())
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    roots.sort();
    Ok(roots)
}

fn diagnostic(path: &str, message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc7007ThreadingBoundary, path, message)
}

#[cfg(test)]
mod tests;
