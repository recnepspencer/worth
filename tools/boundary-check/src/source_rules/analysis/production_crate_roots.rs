//! Production crate discovery across root and application member lanes.
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn crate_roots(root: &Path) -> Result<Vec<String>, String> {
    let mut lanes = vec![PathBuf::from("crates")];
    let workspace_root = root.join("workspaces");
    if workspace_root.is_dir() {
        for entry in
            fs::read_dir(&workspace_root).map_err(|error| format!("read workspaces: {error}"))?
        {
            let entry = entry.map_err(|error| format!("read workspace entry: {error}"))?;
            if entry.path().is_dir() {
                let relative = PathBuf::from("workspaces").join(entry.file_name());
                lanes.push(relative.join("crates"));
                lanes.push(relative.join("apps"));
                lanes.push(relative.join("tools"));
            }
        }
    }
    let mut roots = Vec::new();
    for lane in lanes {
        let absolute = root.join(&lane);
        if !absolute.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&absolute)
            .map_err(|error| format!("read {}: {error}", absolute.display()))?
        {
            let entry = entry.map_err(|error| format!("read crate entry: {error}"))?;
            if entry.path().join("Cargo.toml").is_file() {
                roots.push(
                    lane.join(entry.file_name())
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    roots.sort();
    Ok(roots)
}
