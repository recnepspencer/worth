//! Manifest lint levels that would silence Clippy's disallowed methods.
//!
//! A suppression attribute is audited where it is written, but a crate can
//! also relax a lint for all of its source from its manifest's `[lints]`, or
//! from the `[workspace.lints]` it inherits. Neither may set the lint, or a
//! group that holds it, to `allow`.

use std::fs;
use std::path::Path;

/// The lints that silence Clippy's disallowed methods when allowed, by tool.
const SILENCING: [(&str, &str); 4] = [
    ("clippy", "disallowed_methods"),
    ("clippy", "style"),
    ("clippy", "all"),
    ("rust", "warnings"),
];

/// Every silencing lint the crate at `crate_root` allows, from its own
/// manifest or from the workspace manifest it inherits lints from.
pub(super) fn relaxed(root: &Path, crate_root: &str) -> Result<Vec<String>, String> {
    let crate_dir = root.join(crate_root);
    let manifest = read(&crate_dir.join("Cargo.toml"))?;
    let lints = match manifest.get("lints") {
        Some(lints) if lints.get("workspace").and_then(toml::Value::as_bool) == Some(true) => {
            workspace_lints(root, &crate_dir)?
        }
        lints => lints.cloned(),
    };
    Ok(lints.as_ref().map(relaxed_in).unwrap_or_default())
}

/// The silencing lints a `[lints]` table sets to `allow`.
pub(super) fn relaxed_in(lints: &toml::Value) -> Vec<String> {
    SILENCING
        .iter()
        .filter_map(|(tool, lint)| {
            let level = lints.get(tool)?.get(lint)?;
            let level = level
                .as_str()
                .or_else(|| level.get("level").and_then(toml::Value::as_str))?;
            (level == "allow").then(|| format!("{tool}::{lint}"))
        })
        .collect()
}

/// The `[workspace.lints]` of the nearest manifest above `crate_dir` that
/// declares a workspace, searching no higher than `root`.
fn workspace_lints(root: &Path, crate_dir: &Path) -> Result<Option<toml::Value>, String> {
    for directory in crate_dir.ancestors().skip(1) {
        let manifest = directory.join("Cargo.toml");
        if manifest.is_file() {
            if let Some(workspace) = read(&manifest)?.get("workspace") {
                return Ok(workspace.get("lints").cloned());
            }
        }
        if directory == root {
            break;
        }
    }
    Err(format!(
        "{} inherits workspace lints, but no workspace manifest was found",
        crate_dir.display()
    ))
}

fn read(manifest: &Path) -> Result<toml::Value, String> {
    let text = fs::read_to_string(manifest)
        .map_err(|error| format!("read {}: {error}", manifest.display()))?;
    toml::from_str(&text).map_err(|error| format!("parse {}: {error}", manifest.display()))
}
