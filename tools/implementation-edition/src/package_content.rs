//! Cargo identifies packages; only independently verified content admits an
//! external package into a participating compiler unit.

use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{compiler_context, fingerprint};

mod git;
mod registry;
#[cfg(test)]
pub(crate) mod tests;

pub(crate) fn catalog(metadata: &Value) -> Result<Vec<Value>, String> {
    let lock_path = Path::new(
        metadata["workspace_root"]
            .as_str()
            .ok_or("Cargo workspace missing")?,
    )
    .join("Cargo.lock");
    let lock: toml::Value =
        toml::from_str(&fs::read_to_string(lock_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    metadata["packages"]
        .as_array()
        .ok_or("Cargo packages missing")?
        .iter()
        .map(|package| {
            let manifest = Path::new(
                package["manifest_path"]
                    .as_str()
                    .ok_or("package manifest missing")?,
            );
            let root = fs::canonicalize(manifest.parent().ok_or("package root missing")?)
                .map_err(|e| e.to_string())?;
            let mut entry =
                json!({"name": package["name"], "version": package["version"], "root": root});
            if let Some(source) = package["source"].as_str() {
                let locked = lock["package"]
                    .as_array()
                    .ok_or("locked package list missing")?
                    .iter()
                    .find(|p| {
                        p["name"].as_str() == package["name"].as_str()
                            && p["version"].as_str() == package["version"].as_str()
                            && p.get("source").and_then(toml::Value::as_str) == Some(source)
                    })
                    .ok_or("external package is absent from Cargo.lock")?;
                entry["source"] = json!(source);
                if source.starts_with("registry+") {
                    entry["checksum"] = json!(locked
                        .get("checksum")
                        .and_then(toml::Value::as_str)
                        .ok_or("locked registry checksum missing")?);
                } else if !source.starts_with("git+") {
                    return Err(format!(
                        "external source kind requires qualification: {source}"
                    ));
                }
                let identity = json!([
                    package["name"],
                    package["version"],
                    source,
                    entry["checksum"]
                ]);
                entry["virtual_name"] = json!(format!(
                    "{}-{}-{}",
                    package["name"].as_str().unwrap(),
                    package["version"].as_str().unwrap(),
                    &fingerprint::value("worth.compiler.package.v1", &identity)[..16]
                ));
            }
            Ok(entry)
        })
        .collect()
}

pub(crate) struct VerifiedContent {
    pub(crate) identity: Value,
    pub(crate) inputs: Vec<Value>,
    pub(crate) files: BTreeMap<PathBuf, Vec<u8>>,
}

pub(crate) fn verify(package: &Value, session: &Value) -> Result<VerifiedContent, String> {
    let Some(source) = package["source"].as_str() else {
        return Ok(VerifiedContent {
            identity: Value::Null,
            inputs: Vec::new(),
            files: BTreeMap::new(),
        });
    };
    let root = Path::new(package["root"].as_str().ok_or("package root missing")?);
    let files = if source.starts_with("registry+") {
        registry::files(package)?
    } else {
        git::files(package)?
    };
    let mut inputs = Vec::new();
    for (relative, expected) in &files {
        let path = root.join(relative);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| format!("external content {}: {e}", path.display()))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || fs::read(&path).map_err(|e| e.to_string())? != *expected
        {
            return Err(format!(
                "extracted external content differs from locked archive/tree: {}",
                path.display()
            ));
        }
        let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
        if !path.starts_with(root) {
            return Err("external file escapes verified package root".into());
        }
        inputs.push(json!({"path": path, "identity": compiler_context::normalize_path(&path.to_string_lossy(), session), "digest": fingerprint::bytes(expected), "test": false, "raw": true}));
    }
    let identity = json!({"name": package["name"], "version": package["version"], "source": source, "checksum": package["checksum"]});
    Ok(VerifiedContent {
        identity,
        inputs,
        files,
    })
}

pub(crate) fn contains_input(content: &VerifiedContent, root: &Path, path: &Path) -> bool {
    content.identity.is_null()
        || path
            .strip_prefix(root)
            .is_ok_and(|relative| content.files.contains_key(relative))
}
