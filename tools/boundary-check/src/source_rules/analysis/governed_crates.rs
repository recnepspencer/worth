//! The governed crates under each configured subworkspace's member lane.

use super::crate_modules::GovernedCrate;
use crate::cargo_graph::{normalize_path, package_name_from_manifest};
use crate::config::SubworkspaceConfig;
use std::fs;
use std::path::Path;

pub(super) fn discover_governed_crates(
    root: &Path,
    subworkspaces: &[SubworkspaceConfig],
) -> Result<Vec<GovernedCrate>, String> {
    let mut crates = Vec::new();
    for subworkspace in subworkspaces {
        let member_lane = root.join(&subworkspace.path).join(
            subworkspace
                .member_lane
                .trim_end_matches("/*")
                .trim_end_matches('*')
                .trim_end_matches('/'),
        );
        if !member_lane.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&member_lane)
            .map_err(|e| format!("read member lane {}: {e}", member_lane.display()))?
        {
            let entry = entry.map_err(|e| format!("read member entry: {e}"))?;
            let crate_path = entry.path();
            let manifest = crate_path.join("Cargo.toml");
            if !crate_path.is_dir() || !manifest.is_file() {
                continue;
            }
            let package = package_name_from_manifest(&manifest)?;
            let relative = normalize_path(
                crate_path
                    .strip_prefix(root)
                    .map_err(|e| format!("strip root from {}: {e}", crate_path.display()))?,
            );
            crates.push(GovernedCrate {
                package,
                crate_root: crate_path,
                relative_crate_root: relative,
            });
        }
    }
    crates.sort_by(|a, b| a.package.cmp(&b.package));
    Ok(crates)
}
