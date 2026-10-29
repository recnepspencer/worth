//! Resolve the dirty scope once; a helper edit rechecks its complete owning crate.
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn changed(root: &Path) -> Result<Option<BTreeSet<PathBuf>>, String> {
    if !root.join(".git").exists() {
        return Ok(None);
    }
    let mut paths = BTreeSet::new();
    for args in [
        vec!["diff", "--name-only", "-z", "HEAD", "--"],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }
        for raw in output.stdout.split(|b| *b == 0).filter(|s| !s.is_empty()) {
            let relative = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
            if relative.starts_with("tools/boundary-check/")
                || relative == "workspaces/worth-query/Cargo.toml"
                || relative == "workspaces/worth-query-bank-world/Cargo.toml"
            {
                return Ok(None);
            }
            paths.insert(root.join(relative));
        }
    }
    Ok(Some(paths))
}

pub(super) fn includes(paths: Option<&BTreeSet<PathBuf>>, owner: &Path) -> bool {
    paths.is_none_or(|paths| paths.iter().any(|path| path.starts_with(owner)))
}
