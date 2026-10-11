//! Governed-tree evidence inventory for the rename ratchet.
//!
//! Owns filesystem traversal, derived-directory pruning, symlink/junction
//! boundary observation, encoding policy, and exact text scanning.
//! Returns typed observations only - never final diagnostics or ratchet policy.

use crate::cargo_graph::{normalize_path, normalize_str};
use crate::config::LegacyReferenceRatchetConfig;
use crate::legacy_references::occurrence::LegacyReferenceOccurrence;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

const DERIVED_DIR_NAMES: &[&str] = &["target"];

/// Typed boundary observation discovered during governed traversal.
#[derive(Clone, Debug)]
pub(super) enum BoundaryObservation {
    /// Governed path is a symlink or junction (fail-closed, no follow).
    GovernedSymlinkOrJunction { relative_path: String },
}

/// Complete scan evidence from governed roots.
#[derive(Debug, Default)]
pub(super) struct ObservedInventory {
    pub(super) occurrences: BTreeSet<LegacyReferenceOccurrence>,
    pub(super) boundary_observations: Vec<BoundaryObservation>,
}

pub(super) fn collect_governed_inventory(
    root: &Path,
    config: &LegacyReferenceRatchetConfig,
) -> Result<ObservedInventory, String> {
    let exclude_paths: HashSet<String> = config
        .exclude_paths
        .iter()
        .map(|path| normalize_str(path))
        .collect();
    let mut inventory = ObservedInventory::default();

    for governed_root in &config.governed_roots {
        let governed_path = root.join(governed_root);
        if !governed_path.exists() {
            continue;
        }
        walk_path(
            root,
            &governed_path,
            &config.forbidden_fragments,
            &exclude_paths,
            &mut inventory,
        )?;
    }

    Ok(inventory)
}

fn walk_path(
    root: &Path,
    path: &Path,
    forbidden_fragments: &[String],
    exclude_paths: &HashSet<String>,
    inventory: &mut ObservedInventory,
) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("stat {}: {error}", path.display()))?;

    // Fail closed: governed symlinks/junctions can hide or re-enter content.
    if metadata.file_type().is_symlink() {
        let relative = relative_path(root, path).unwrap_or_else(|_| path.display().to_string());
        inventory
            .boundary_observations
            .push(BoundaryObservation::GovernedSymlinkOrJunction {
                relative_path: relative,
            });
        return Ok(());
    }

    if metadata.is_dir() {
        return walk_directory(root, path, forbidden_fragments, exclude_paths, inventory);
    }

    if metadata.is_file() {
        inventory_file(root, path, forbidden_fragments, exclude_paths, inventory)?;
    }

    Ok(())
}

fn walk_directory(
    root: &Path,
    path: &Path,
    forbidden_fragments: &[String],
    exclude_paths: &HashSet<String>,
    inventory: &mut ObservedInventory,
) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if DERIVED_DIR_NAMES.contains(&name) {
        return Ok(());
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("read dir {}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read dir entry under {}: {error}", path.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        walk_path(
            root,
            &entry.path(),
            forbidden_fragments,
            exclude_paths,
            inventory,
        )?;
    }
    Ok(())
}

fn inventory_file(
    root: &Path,
    path: &Path,
    forbidden_fragments: &[String],
    exclude_paths: &HashSet<String>,
    inventory: &mut ObservedInventory,
) -> Result<(), String> {
    let relative = relative_path(root, path)?;
    if exclude_paths.contains(&relative) {
        return Ok(());
    }

    let bytes = crate::source_documents::read(path)
        .map_err(|error| format!("read file {}: {error}", path.display()))?;
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Ok(());
    };

    for (line_idx, line) in text.lines().enumerate() {
        for fragment in forbidden_fragments {
            // Exact occurrence inventory: every non-overlapping hit on the line.
            let mut start = 0usize;
            while let Some(rel) = line[start..].find(fragment.as_str()) {
                let column = start + rel;
                inventory.occurrences.insert(LegacyReferenceOccurrence::new(
                    relative.clone(),
                    format!("{}:{}", line_idx + 1, column + 1),
                    fragment.clone(),
                ));
                start = column + fragment.len();
                if start >= line.len() {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| format!("strip root from {}: {error}", path.display()))?;
    Ok(normalize_path(relative))
}
