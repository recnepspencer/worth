use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

fn git(root: &Path, arguments: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .arg("--no-replace-objects")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "locked Git content: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

pub(super) fn files(package: &Value) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
    let root = Path::new(package["root"].as_str().unwrap());
    let revision = package["source"]
        .as_str()
        .and_then(|s| s.rsplit_once('#'))
        .map(|(_, r)| r)
        .ok_or("locked Git revision missing")?;
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Git source requires a full locked commit".into());
    }
    let top = git(root, &["rev-parse", "--show-toplevel"])?;
    let top = std::fs::canonicalize(String::from_utf8(top).map_err(|e| e.to_string())?.trim())
        .map_err(|e| e.to_string())?;
    let relative = root
        .strip_prefix(&top)
        .map_err(|_| "Git package root escapes tree")?;
    let prefix = relative.to_string_lossy().replace('\\', "/");
    let tree = git(
        &top,
        &[
            "ls-tree",
            "-r",
            "-z",
            revision,
            "--",
            if prefix.is_empty() { "." } else { &prefix },
        ],
    )?;
    let mut result = BTreeMap::new();
    for entry in tree.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let entry = std::str::from_utf8(entry).map_err(|e| e.to_string())?;
        let (header, path) = entry.split_once('\t').ok_or("invalid Git tree entry")?;
        if !header.starts_with("100644 blob ") && !header.starts_with("100755 blob ") {
            return Err("Git links/submodules require qualification".into());
        }
        let local = if prefix.is_empty() {
            path
        } else {
            path.strip_prefix(&(prefix.clone() + "/"))
                .ok_or("Git tree path escapes package")?
        };
        result.insert(
            PathBuf::from(local),
            git(&top, &["show", &format!("{revision}:{path}")])?,
        );
    }
    if result.is_empty() {
        return Err("locked Git package tree is empty".into());
    }
    Ok(result)
}
