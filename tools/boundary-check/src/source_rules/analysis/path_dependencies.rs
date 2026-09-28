//! Resolve inspectable library dependencies from a governed crate's Cargo.toml.
//!
//! Completeness for BC7001 authority identity:
//! - top-level `[dependencies]`
//! - workspace-inherited `{ workspace = true }` rows (path taken from
//!   `[workspace.dependencies]`)
//! - target-specific `[target.'cfg(...)'.dependencies]` tables (all targets,
//!   fail-closed so inactive cfgs cannot hide a renamed authority export)
//! - package-key Rust idents (the Cargo dependency key is the extern name)
//!
//! Only path-backed sources are inspectable. Version, git, registry, and other
//! non-path sources are fail-closed errors: they are ordinary Cargo inputs that
//! can rename platform markers, and silent omission would admit forged
//! authority. Callers must not default resolution failures to an empty index.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Rust extern idents whose Cargo package identity is `package`.
pub(super) fn dependency_idents_for_package(
    crate_root: &Path,
    package: &str,
) -> Result<std::collections::BTreeSet<String>, String> {
    let roots = path_dependency_roots(crate_root)?;
    let mut idents = std::collections::BTreeSet::new();
    for (ident, root) in roots {
        let manifest = root.join("Cargo.toml");
        let actual = crate::cargo_graph::package_name_from_manifest(&manifest)?;
        if actual == package {
            idents.insert(ident.replace('-', "_"));
        }
    }
    Ok(idents)
}

/// Map from Rust crate ident (`worth_schema_authgate`) to dependency crate root.
pub(super) fn path_dependency_roots(
    crate_root: &Path,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let manifest_path = crate_root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("read {}: {e}", manifest_path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;

    let workspace_root = find_workspace_root(crate_root)?;
    let workspace_deps = load_workspace_dependency_table(&workspace_root)?;

    let mut roots = BTreeMap::new();
    if let Some(dependencies) = value.get("dependencies").and_then(|v| v.as_table()) {
        collect_dep_table(
            crate_root,
            &workspace_root,
            &workspace_deps,
            dependencies,
            &mut roots,
        )?;
    }
    // Target-specific deps: inventory every cfg branch fail-closed.
    if let Some(targets) = value.get("target").and_then(|v| v.as_table()) {
        for (target_key, target_val) in targets {
            let Some(deps) = target_val.get("dependencies").and_then(|v| v.as_table()) else {
                continue;
            };
            collect_dep_table(
                crate_root,
                &workspace_root,
                &workspace_deps,
                deps,
                &mut roots,
            )
            .map_err(|e| format!("target `{target_key}` dependencies: {e}"))?;
        }
    }
    Ok(roots)
}

/// Map from Rust crate ident to crate root for path-backed library dependencies only.
///
/// Read-only navigation (facade doc resolution) uses this: a registry, git, or
/// version dependency cannot hold a workspace definition, so such rows are
/// skipped instead of failing. Callers fail closed when a path they must follow
/// names a crate this map does not contain.
pub(crate) fn path_backed_dependency_roots(
    crate_root: &Path,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let manifest_path = crate_root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("read {}: {e}", manifest_path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;
    let workspace_root = find_workspace_root(crate_root)?;
    let workspace_deps = load_workspace_dependency_table(&workspace_root)?;
    let mut tables = Vec::new();
    tables.extend(value.get("dependencies").and_then(|v| v.as_table()));
    if let Some(targets) = value.get("target").and_then(|v| v.as_table()) {
        tables.extend(
            targets
                .values()
                .filter_map(|target| target.get("dependencies").and_then(|v| v.as_table())),
        );
    }
    let mut roots = BTreeMap::new();
    for (dep_key, spec) in tables.into_iter().flatten() {
        if let Ok((ident, dep_root)) =
            resolve_dependency_entry(crate_root, &workspace_root, &workspace_deps, dep_key, spec)
        {
            roots.insert(ident, dep_root);
        }
    }
    Ok(roots)
}

fn collect_dep_table(
    crate_root: &Path,
    workspace_root: &Path,
    workspace_deps: &BTreeMap<String, toml::Value>,
    dependencies: &toml::map::Map<String, toml::Value>,
    roots: &mut BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    for (dep_key, spec) in dependencies {
        let (ident, dep_root) =
            resolve_dependency_entry(crate_root, workspace_root, workspace_deps, dep_key, spec)?;
        roots.insert(ident.clone(), dep_root.clone());
        roots.insert(dep_key.clone(), dep_root);
    }
    Ok(())
}

fn resolve_dependency_entry(
    crate_root: &Path,
    workspace_root: &Path,
    workspace_deps: &BTreeMap<String, toml::Value>,
    dep_key: &str,
    spec: &toml::Value,
) -> Result<(String, PathBuf), String> {
    match spec {
        // `dep = "1.0"` version-only: ordinary non-path source — fail closed.
        toml::Value::String(version) => Err(non_path_source_error(
            dep_key,
            &format!("version string `{version}`"),
        )),
        toml::Value::Table(table) => {
            // Workspace inheritance: path lives in [workspace.dependencies].
            if table
                .get("workspace")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let ws_spec = workspace_deps.get(dep_key).ok_or_else(|| {
                    format!(
                        "dependency `{dep_key}` is workspace = true but missing from \
[workspace.dependencies] under {}",
                        workspace_root.display()
                    )
                })?;
                return resolve_path_spec(workspace_root, dep_key, ws_spec);
            }
            resolve_path_spec(crate_root, dep_key, spec)
        }
        other => Err(non_path_source_error(
            dep_key,
            &format!("unsupported dependency value kind `{}`", value_kind(other)),
        )),
    }
}

fn resolve_path_spec(
    base_root: &Path,
    dep_key: &str,
    spec: &toml::Value,
) -> Result<(String, PathBuf), String> {
    let Some(path) = dependency_path_spec(spec) else {
        return Err(non_path_source_error(dep_key, &source_kind_label(spec)));
    };
    let dep_root = normalize_dep_root(base_root, Path::new(path))?;
    let rust_ident = dep_key.replace('-', "_");
    Ok((rust_ident, dep_root))
}

fn dependency_path_spec(spec: &toml::Value) -> Option<&str> {
    match spec {
        toml::Value::Table(table) => table.get("path").and_then(|v| v.as_str()),
        _ => None,
    }
}

fn non_path_source_error(dep_key: &str, source_kind: &str) -> String {
    format!(
        "dependency `{dep_key}` uses non-path source ({source_kind}); BC7001 requires \
inspectable path dependencies so renamed platform authority traits cannot be \
silently omitted from the sealed-export index"
    )
}

fn source_kind_label(spec: &toml::Value) -> String {
    let Some(table) = spec.as_table() else {
        return value_kind(spec).to_owned();
    };
    if table.contains_key("git") {
        return "git".to_owned();
    }
    if table.contains_key("registry") {
        return "registry".to_owned();
    }
    if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
        return format!("version `{version}` without path");
    }
    "table without path".to_owned()
}

fn value_kind(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::String(_) => "string",
        toml::Value::Integer(_) => "integer",
        toml::Value::Float(_) => "float",
        toml::Value::Boolean(_) => "boolean",
        toml::Value::Datetime(_) => "datetime",
        toml::Value::Array(_) => "array",
        toml::Value::Table(_) => "table",
    }
}

fn find_workspace_root(crate_root: &Path) -> Result<PathBuf, String> {
    // Prefer nearest ancestor Cargo.toml that defines [workspace].
    let mut dir = crate_root.to_path_buf();
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            if let Ok(text) = fs::read_to_string(&candidate) {
                if let Ok(value) = toml::from_str::<toml::Value>(&text) {
                    if value.get("workspace").is_some() {
                        return Ok(dir);
                    }
                }
            }
        }
        if !dir.pop() {
            // Fall back to the crate itself (standalone package / virtual root).
            return Ok(crate_root.to_path_buf());
        }
    }
}

fn load_workspace_dependency_table(
    workspace_root: &Path,
) -> Result<BTreeMap<String, toml::Value>, String> {
    let manifest = workspace_root.join("Cargo.toml");
    if !manifest.is_file() {
        return Ok(BTreeMap::new());
    }
    let text =
        fs::read_to_string(&manifest).map_err(|e| format!("read {}: {e}", manifest.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", manifest.display()))?;
    let Some(table) = value
        .get("workspace")
        .and_then(|ws| ws.get("dependencies"))
        .and_then(|d| d.as_table())
    else {
        return Ok(BTreeMap::new());
    };
    Ok(table.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

fn normalize_dep_root(base_root: &Path, rel: &Path) -> Result<PathBuf, String> {
    let joined = base_root.join(rel);
    let canonical = fs::canonicalize(&joined).unwrap_or(joined);
    let has_lib = canonical.join("src/lib.rs").is_file()
        || lib_path_from_manifest(&canonical).is_some_and(|p| p.is_file());
    if !has_lib {
        return Err(format!(
            "path dependency root {} is missing a library target (src/lib.rs or [lib].path)",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn lib_path_from_manifest(dep_root: &Path) -> Option<PathBuf> {
    let text = fs::read_to_string(dep_root.join("Cargo.toml")).ok()?;
    let value: toml::Value = toml::from_str(&text).ok()?;
    let rel = value
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(|p| p.as_str())?;
    Some(dep_root.join(rel))
}
