use crate::cargo_graph::{normalize_path, normalize_str, package_name_from_manifest};
use crate::config::{BornCrateConfig, SeedSkeletonConfig, SubworkspaceConfig};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use syn::{Item, ItemMod};

pub(crate) fn validate_seed_crate_contracts(
    root: &Path,
    born_crates: &[BornCrateConfig],
    seed_skeletons: &[SeedSkeletonConfig],
    subworkspaces: &[SubworkspaceConfig],
) -> Result<Vec<Diagnostic>, String> {
    let expected_paths: BTreeSet<_> = born_crates
        .iter()
        .map(|born| normalize_str(&born.path))
        .collect();
    let actual_paths = discover_born_crates(root, subworkspaces)?;
    let mut diagnostics = Vec::new();

    if actual_paths != expected_paths {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5003SeedContractViolation,
            "road1-born-crates",
            format!(
                "born crate set mismatch: expected {:?}, found {:?}",
                expected_paths, actual_paths
            ),
        ));
    }

    for born in born_crates {
        let manifest_path = root.join(&born.path).join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let actual_package = package_name_from_manifest(&manifest_path)?;
        if actual_package != born.package {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc5003SeedContractViolation,
                manifest_path.display().to_string(),
                format!(
                    "born crate package mismatch: expected {}, found {}",
                    born.package, actual_package
                ),
            ));
        }
    }

    for skeleton in seed_skeletons {
        diagnostics.extend(validate_seed_skeleton(root, skeleton)?);
    }

    Ok(diagnostics)
}

fn discover_born_crates(
    root: &Path,
    subworkspaces: &[SubworkspaceConfig],
) -> Result<BTreeSet<String>, String> {
    let mut born_crates = BTreeSet::new();

    for workspace in subworkspaces {
        let crates_path = root.join(&workspace.path).join("crates");

        for crate_dir in fs::read_dir(&crates_path)
            .map_err(|e| format!("read crates lane {}: {e}", crates_path.display()))?
        {
            let crate_dir = crate_dir.map_err(|e| format!("read born crate entry: {e}"))?;
            let crate_path = crate_dir.path();
            if !crate_path.is_dir() || !crate_path.join("Cargo.toml").is_file() {
                continue;
            }
            let relative = crate_path
                .strip_prefix(root)
                .map_err(|e| format!("strip root prefix from {}: {e}", crate_path.display()))?;
            born_crates.insert(normalize_path(relative));
        }
    }

    Ok(born_crates)
}

fn validate_seed_skeleton(
    root: &Path,
    skeleton: &SeedSkeletonConfig,
) -> Result<Vec<Diagnostic>, String> {
    let crate_root = root.join(&skeleton.path);
    let mut diagnostics = Vec::new();

    if !crate_root.is_dir() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5003SeedContractViolation,
            crate_root.display().to_string(),
            "missing seed crate root",
        ));
        return Ok(diagnostics);
    }

    let package = package_name_from_manifest(&crate_root.join("Cargo.toml"))?;
    if package != skeleton.package {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5003SeedContractViolation,
            crate_root.display().to_string(),
            format!("expected package {}, found {}", skeleton.package, package),
        ));
    }

    let actual_entries = collect_entries(&crate_root)?;
    let expected_entries: BTreeSet<_> = skeleton
        .allowed_entries
        .iter()
        .map(|entry| normalize_str(entry))
        .collect();
    if actual_entries != expected_entries {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5003SeedContractViolation,
            crate_root.display().to_string(),
            format!(
                "seed crate skeleton mismatch: expected {:?}, found {:?}",
                expected_entries, actual_entries
            ),
        ));
    }

    if let Some(message) = verify_lib_rs_contract(&crate_root.join(&skeleton.lib_rs))? {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5003SeedContractViolation,
            crate_root.join(&skeleton.lib_rs).display().to_string(),
            message,
        ));
    }
    if let Some(message) = verify_facade_rs_contract(&crate_root.join(&skeleton.facade_rs))? {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc5003SeedContractViolation,
            crate_root.join(&skeleton.facade_rs).display().to_string(),
            message,
        ));
    }

    Ok(diagnostics)
}

fn collect_entries(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut entries = BTreeSet::new();
    collect_entries_recursive(root, root, &mut entries)?;
    Ok(entries)
}

fn collect_entries_recursive(
    root: &Path,
    current: &Path,
    entries: &mut BTreeSet<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|e| format!("read seed crate path {}: {e}", current.display()))?
    {
        let entry = entry.map_err(|e| format!("read seed crate entry: {e}"))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|e| format!("strip crate root prefix from {}: {e}", path.display()))?;
        entries.insert(normalize_path(relative));

        if path.is_dir() {
            collect_entries_recursive(root, &path, entries)?;
        }
    }

    Ok(())
}

fn verify_lib_rs_contract(path: &Path) -> Result<Option<String>, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("read source file {}: {e}", path.display()))?;
    let syntax = syn::parse_file(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if syntax.items.is_empty() {
        return Ok(Some("lib.rs must not be empty".to_owned()));
    }

    let mut facade_exports = 0usize;
    for item in syntax.items {
        match item {
            Item::Mod(ItemMod {
                vis: syn::Visibility::Public(_),
                ident,
                content: None,
                ..
            }) if ident == "facade" => {
                facade_exports += 1;
            }
            Item::Mod(ItemMod {
                vis: syn::Visibility::Inherited,
                content: None,
                ..
            }) => {}
            Item::Use(item_use) if matches!(item_use.vis, syn::Visibility::Public(_)) => {
                return Ok(Some("lib.rs must export only the facade module".to_owned()));
            }
            _ => {
                return Ok(Some(
                    "lib.rs contains non-visibility wiring outside the facade contract".to_owned(),
                ));
            }
        }
    }

    if facade_exports != 1 {
        return Ok(Some(
            "lib.rs must contain exactly one public facade module declaration".to_owned(),
        ));
    }

    Ok(None)
}

fn verify_facade_rs_contract(path: &Path) -> Result<Option<String>, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("read source file {}: {e}", path.display()))?;
    let syntax = syn::parse_file(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if syntax.items.is_empty() {
        return Ok(Some("facade.rs must not be empty".to_owned()));
    }

    for item in syntax.items {
        match item {
            Item::Use(item_use) if matches!(item_use.vis, syn::Visibility::Public(_)) => {}
            _ => {
                return Ok(Some(
                    "facade.rs must aggregate public exports only".to_owned(),
                ));
            }
        }
    }

    Ok(None)
}
