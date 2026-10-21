//! Physical source locations relative to the consuming crate, including siblings.

use crate::cargo_graph::normalize_path;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(super) fn relative_source_path(
    crate_root: &Path,
    source_path: &Path,
) -> Result<String, String> {
    let root = fs::canonicalize(crate_root)
        .map_err(|error| format!("resolve crate root {}: {error}", crate_root.display()))?;
    let source = fs::canonicalize(source_path)
        .map_err(|error| format!("resolve source location {}: {error}", source_path.display()))?;
    let root_components = root.components().collect::<Vec<_>>();
    let source_components = source.components().collect::<Vec<_>>();
    let root_anchor = root_components
        .iter()
        .take_while(|part| !matches!(part, Component::Normal(_)));
    let source_anchor = source_components
        .iter()
        .take_while(|part| !matches!(part, Component::Normal(_)));
    if !root_anchor.eq(source_anchor) {
        return Err(format!(
            "source location {} is not representable relative to crate root {}: incompatible path prefix or root",
            source.display(), root.display()
        ));
    }
    let shared = root_components
        .iter()
        .zip(&source_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for component in &root_components[shared..] {
        if !matches!(component, Component::Normal(_)) {
            return Err(format!("noncanonical crate root {}", root.display()));
        }
        relative.push("..");
    }
    for component in &source_components[shared..] {
        let Component::Normal(name) = component else {
            return Err(format!("noncanonical source location {}", source.display()));
        };
        relative.push(name);
    }
    Ok(normalize_path(&relative))
}
