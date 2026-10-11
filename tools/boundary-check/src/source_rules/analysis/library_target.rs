//! Resolve the Rust source root selected by a crate's Cargo library target.

use std::path::{Path, PathBuf};

pub(super) fn resolve_lib_source_path(crate_root: &Path) -> Result<PathBuf, String> {
    let manifest_path = crate_root.join("Cargo.toml");
    let text = crate::source_documents::read_to_string(&manifest_path)
        .map_err(|error| format!("read {}: {error}", manifest_path.display()))?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
    let relative = value
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(|path| path.as_str())
        .unwrap_or("src/lib.rs");
    Ok(crate_root.join(relative))
}
