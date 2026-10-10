use std::{
    fs,
    path::{Path, PathBuf},
};

/// Enumerate source/manifests without traversing derived build trees.
pub(crate) fn files_under(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect(root, &mut files)?;
    Ok(files)
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if matches!(
                entry.file_name().to_str(),
                Some("target" | ".git" | "node_modules" | "dist")
            ) {
                continue;
            }
            collect(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs")
            || entry.file_name() == "Cargo.toml"
        {
            files.push(path);
        }
    }
    Ok(())
}
