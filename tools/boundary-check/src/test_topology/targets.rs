//! Cargo integration target discovery, including implicit directory targets.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) struct Target {
    pub(super) name: String,
    pub(super) path: PathBuf,
    pub(super) explicit: bool,
    pub(super) harness: bool,
    pub(super) tested: bool,
}

pub(super) fn discover(root: &Path, value: &toml::Value) -> Result<Vec<Target>, String> {
    let mut targets = BTreeMap::new();
    if value
        .get("package")
        .and_then(|p| p.get("autotests"))
        .and_then(|v| v.as_bool())
        != Some(false)
        && root.join("tests").is_dir()
    {
        for entry in std::fs::read_dir(root.join("tests")).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let source = if path.is_dir() {
                path.join("main.rs")
            } else {
                path.clone()
            };
            if !source.is_file() || source.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let name = if path.is_dir() {
                path.file_name()
            } else {
                path.file_stem()
            }
            .unwrap()
            .to_string_lossy()
            .into_owned();
            targets.insert(
                name.clone(),
                Target {
                    name,
                    path: source,
                    explicit: false,
                    harness: true,
                    tested: true,
                },
            );
        }
    }
    for declaration in value
        .get("test")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let name = declaration
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("[[test]] needs name")?;
        let path = declaration
            .get("path")
            .and_then(|p| p.as_str())
            .map(|p| root.join(p))
            .or_else(|| targets.get(name).map(|t| t.path.clone()))
            .unwrap_or_else(|| root.join(format!("tests/{name}.rs")));
        targets.insert(
            name.into(),
            Target {
                name: name.into(),
                path,
                explicit: true,
                harness: declaration
                    .get("harness")
                    .and_then(|h| h.as_bool())
                    .unwrap_or(true),
                tested: declaration
                    .get("test")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            },
        );
    }
    Ok(targets.into_values().collect())
}
