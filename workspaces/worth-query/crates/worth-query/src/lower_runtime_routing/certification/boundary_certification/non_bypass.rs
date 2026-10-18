use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::identity::hash_parts;

use super::public_surface::{
    worth_query_lower_runtime_public_surface_inventory, WorthQueryLowerRuntimePublicSurfaceKind,
};
use crate::lower_runtime_routing::worth_query_lower_runtime_direct_import_audit;

const LOWER_RUNTIME_IMPORT_MARKERS: &[&str] = &[
    "worth_runtime_bridge::facade",
    "worth_relational::facade",
    "worth_signal::facade",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeNonBypassAudit {
    route_public_surface_digest: String,
    route_non_bypass_digest: String,
    checked_file_count: usize,
}

impl WorthQueryLowerRuntimeNonBypassAudit {
    pub(crate) fn new(
        route_public_surface_digest: String,
        route_non_bypass_digest: String,
        checked_file_count: usize,
    ) -> Self {
        Self {
            route_public_surface_digest,
            route_non_bypass_digest,
            checked_file_count,
        }
    }

    pub fn route_public_surface_digest(&self) -> &str {
        &self.route_public_surface_digest
    }

    pub fn route_non_bypass_digest(&self) -> &str {
        &self.route_non_bypass_digest
    }

    pub fn checked_file_count(&self) -> usize {
        self.checked_file_count
    }
}

pub fn certify_lower_runtime_non_bypass() -> Result<WorthQueryLowerRuntimeNonBypassAudit, String> {
    let inventory = worth_query_lower_runtime_public_surface_inventory();
    let workspace_root = workspace_root()?;
    let mut violations = Vec::new();
    let mut checked_files = 0usize;

    for (relative, allow_imports) in routed_surface_scan_targets() {
        checked_files +=
            scan_surface_target(&workspace_root, relative, allow_imports, &mut violations)?;
    }

    for row in worth_query_lower_runtime_direct_import_audit().rows() {
        checked_files +=
            scan_allowed_query_boundary(&workspace_root, row.module_path(), &mut violations)?;
    }

    if !violations.is_empty() {
        return Err(violations.join("\n"));
    }

    let route_public_surface_digest = inventory.public_surface_digest();
    let route_non_bypass_digest = hash_parts(&[
        route_public_surface_digest.clone(),
        checked_files.to_string(),
        "query-runtime-routed-surfaces:no-lower-runtime-imports".to_string(),
    ]);
    Ok(WorthQueryLowerRuntimeNonBypassAudit::new(
        route_public_surface_digest,
        route_non_bypass_digest,
        checked_files,
    ))
}

fn routed_surface_scan_targets() -> Vec<(&'static str, bool)> {
    let mut seen = BTreeMap::new();
    for row in worth_query_lower_runtime_public_surface_inventory().rows() {
        let allow_imports = matches!(
            row.surface_kind(),
            WorthQueryLowerRuntimePublicSurfaceKind::AllowedBoundaryAdapter
                | WorthQueryLowerRuntimePublicSurfaceKind::RuntimeBackendBoundary
        );
        seen.entry(row.implementation_path())
            .and_modify(|existing| *existing |= allow_imports)
            .or_insert(allow_imports);
    }
    seen.into_iter().collect()
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "could not derive workspace root from CARGO_MANIFEST_DIR".to_string())
}

fn scan_allowed_query_boundary(
    workspace_root: &Path,
    relative: &str,
    violations: &mut Vec<String>,
) -> Result<usize, String> {
    if relative.ends_with("/*") {
        return scan_tree(
            workspace_root,
            relative.trim_end_matches("/*"),
            None,
            violations,
        );
    }
    scan_surface_file(workspace_root, relative, true, violations)
}

fn scan_tree(
    workspace_root: &Path,
    relative_root: &str,
    allowed_prefix: Option<&str>,
    violations: &mut Vec<String>,
) -> Result<usize, String> {
    let root = workspace_root.join(relative_root);
    let mut count = 0usize;
    for path in rust_files_under(&root)? {
        count += 1;
        let relative = slash_path(
            path.strip_prefix(workspace_root)
                .map_err(|error| error.to_string())?,
        );
        let allow_imports = allowed_prefix
            .map(|prefix| relative.starts_with(prefix))
            .unwrap_or(true);
        scan_file_contents(&path, &relative, allow_imports, violations)?;
    }
    Ok(count)
}

fn scan_surface_file(
    workspace_root: &Path,
    relative: &str,
    allow_imports: bool,
    violations: &mut Vec<String>,
) -> Result<usize, String> {
    let path = workspace_root.join(relative);
    scan_file_contents(&path, relative, allow_imports, violations)?;
    Ok(1)
}

fn scan_surface_target(
    workspace_root: &Path,
    relative: &str,
    allow_imports: bool,
    violations: &mut Vec<String>,
) -> Result<usize, String> {
    if relative.ends_with("/*") {
        return scan_tree(
            workspace_root,
            relative.trim_end_matches("/*"),
            None,
            violations,
        );
    }
    scan_surface_file(workspace_root, relative, allow_imports, violations)
}

fn scan_file_contents(
    path: &Path,
    relative: &str,
    allow_imports: bool,
    violations: &mut Vec<String>,
) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    for (line_number, line) in source.lines().enumerate() {
        if let Some(marker) = LOWER_RUNTIME_IMPORT_MARKERS
            .iter()
            .find(|marker| line.contains(**marker))
            && !allow_imports
        {
            violations.push(format!(
                "{relative}:{} imports `{marker}` outside the declared routed boundary",
                line_number + 1
            ));
        }
    }
    Ok(())
}

fn rust_files_under(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_files_under(&path)?);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(files)
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod non_bypass_tests;
