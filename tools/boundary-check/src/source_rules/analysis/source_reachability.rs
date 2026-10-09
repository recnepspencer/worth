//! Reject production Rust sources that no supported Cargo target can compile.

use super::crate_modules::{GovernedCrate, ModuleGraph, ModuleNode};
use crate::cargo_graph::normalize_path;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXEMPTIONS: &str = "tools/boundary-check/config/generated_source_exemptions.txt";

pub(crate) fn enforce_workspace_source_reachability(
    root: &Path,
    relative_workspace: &str,
) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for governed in super::workspace_crates::discover_workspace_crates(root, relative_workspace)? {
        let library = super::crate_modules::parse_crate_modules(&governed)?;
        let additional = super::crate_modules::parse_additional_production_targets(&governed)?;
        diagnostics.extend(enforce_source_reachability(
            root,
            &governed,
            &library,
            &additional,
        )?);
    }
    Ok(diagnostics)
}

pub(super) fn enforce_source_reachability(
    root: &Path,
    governed: &GovernedCrate,
    library: &ModuleGraph,
    additional_targets: &[ModuleNode],
) -> Result<Vec<Diagnostic>, String> {
    let reachable = reachable_sources(&governed.crate_root, library, additional_targets)?;
    let exemptions = load_exemptions(root)?;
    let mut sources = Vec::new();
    collect_rust_sources(&governed.crate_root.join("src"), &mut sources)?;
    sources.sort();

    let mut diagnostics = Vec::new();
    for source in sources {
        let relative_to_workspace = relative_source(root, &source)?;
        if reachable.contains(&source_identity(&source)?)
            || exemptions.contains(&relative_to_workspace)
        {
            continue;
        }
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc7003SourceReachability,
            format!("{}::{relative_to_workspace}", governed.package),
            format!(
                "production Rust source `{relative_to_workspace}` is absent from every compiled \
library, feature/platform module, bin, example, bench, and explicit Cargo target graph"
            ),
        ));
    }
    Ok(diagnostics)
}

fn reachable_sources(
    crate_root: &Path,
    library: &ModuleGraph,
    additional_targets: &[ModuleNode],
) -> Result<BTreeSet<PathBuf>, String> {
    library
        .modules
        .values()
        .chain(additional_targets)
        .flat_map(|node| node.relative_source.split(';'))
        .map(|relative| source_identity(&crate_root.join(relative)))
        .collect()
}

fn source_identity(source: &Path) -> Result<PathBuf, String> {
    // Rustc resolves the selected file on disk, including parent components
    // and symlinks. Compare that identity without changing diagnostic paths.
    fs::canonicalize(source)
        .map_err(|error| format!("resolve source identity {}: {error}", source.display()))
}

fn load_exemptions(root: &Path) -> Result<BTreeSet<String>, String> {
    let path = root.join(EXEMPTIONS);
    if !path.is_file() {
        return Ok(BTreeSet::new());
    }
    let text =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut exemptions = BTreeSet::new();
    for (index, raw) in text.lines().enumerate() {
        let entry = raw.trim();
        if entry.is_empty() || entry.starts_with('#') {
            continue;
        }
        if !entry.ends_with(".rs") || !entry.contains("/src/") {
            return Err(format!(
                "{}:{} generated-source exemption must be a workspace-relative production .rs path",
                path.display(),
                index + 1
            ));
        }
        exemptions.insert(entry.replace('\\', "/"));
    }
    Ok(exemptions)
}

fn collect_rust_sources(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("read source directory {}: {error}", directory.display()))?
    {
        let path = entry
            .map_err(|error| format!("read source directory entry: {error}"))?
            .path();
        if path.is_dir() {
            collect_rust_sources(&path, paths)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
    Ok(())
}

fn relative_source(root: &Path, source: &Path) -> Result<String, String> {
    source
        .strip_prefix(root)
        .map(normalize_path)
        .map_err(|error| format!("strip root from {}: {error}", source.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orphan_is_rejected_while_cfg_and_path_modules_are_reachable() {
        let root =
            std::env::temp_dir().join(format!("worth-source-reachability-{}", std::process::id()));
        let crate_root = root.join("workspaces/worth-ui/crates/worth-test");
        fs::create_dir_all(crate_root.join("src/group")).unwrap();
        fs::write(
            crate_root.join("Cargo.toml"),
            "[package]\nname='worth-test'\nversion='0.1.0'\nedition='2021'\n",
        )
        .unwrap();
        fs::write(
            crate_root.join("src/lib.rs"),
            "#[cfg(target_os = \"windows\")] mod platform;\nmod group;\n",
        )
        .unwrap();
        fs::write(crate_root.join("src/platform.rs"), "pub fn platform() {}\n").unwrap();
        fs::write(
            crate_root.join("src/group/mod.rs"),
            "#[path = \"../selected.rs\"] mod custom;\n",
        )
        .unwrap();
        fs::write(crate_root.join("src/selected.rs"), "pub fn selected() {}\n").unwrap();
        fs::write(crate_root.join("src/orphan.rs"), "pub fn orphan() {}\n").unwrap();
        let governed = GovernedCrate {
            package: "worth-test".to_owned(),
            crate_root: crate_root.clone(),
            relative_crate_root: "workspaces/worth-ui/crates/worth-test".to_owned(),
        };
        let graph = super::super::crate_modules::parse_crate_modules(&governed).unwrap();
        let additional =
            super::super::crate_modules::parse_additional_source_targets(&governed).unwrap();
        let diagnostics =
            enforce_source_reachability(&root, &governed, &graph, &additional).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].subject().ends_with("src/orphan.rs"));
        fs::remove_dir_all(root).unwrap();
    }
}
