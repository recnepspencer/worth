//! Application of the two admitted guidance output classes.
//!
//! This is a local, quiescent-checkout writer, not a hostile-filesystem sandbox.
//! Links/reparse points and changed observations refuse; same-directory rename
//! publishes complete files. Concurrent adversarial directory replacement is not
//! supported. No cross-platform race-safe confinement claim is made.
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

#[derive(Eq, PartialEq)]
pub(crate) enum State {
    Missing,
    Directory,
    File(Vec<u8>),
}
pub(crate) struct Plan {
    root: PathBuf,
    observed: BTreeMap<PathBuf, State>,
    writes: BTreeMap<PathBuf, Vec<u8>>,
    removes: Vec<PathBuf>,
}
/// Inspect each component without following links, including Windows junctions.
pub(crate) fn inspect(root: &Path, path: &Path, missing: bool) -> Result<State, String> {
    let relative = path.strip_prefix(root).map_err(|_| "output escapes root")?;
    if !relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err("non-normal output path".into());
    }
    let mut current = root.to_owned();
    for component in relative.components() {
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err("output link is unsupported".into());
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if metadata.file_attributes() & 0x400 != 0 {
                        return Err("output reparse point is unsupported".into());
                    }
                }
                if current != path && !metadata.is_dir() {
                    return Err("output parent is not a directory".into());
                }
                if current == path {
                    return if metadata.is_dir() {
                        Ok(State::Directory)
                    } else if metadata.is_file() {
                        Ok(State::File(std::fs::read(path).map_err(|e| e.to_string())?))
                    } else {
                        Err("unsupported output file kind".into())
                    };
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && missing => {
                return Ok(State::Missing)
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("output must not be repository root".into())
}
impl Plan {
    pub(crate) fn new(
        root: &Path,
        documents: BTreeMap<PathBuf, String>,
        skills: Option<crate::derived_skills::Plan>,
    ) -> Result<Self, String> {
        let original = root;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let mut writes = BTreeMap::new();
        for (path, text) in documents {
            let path = normalize(original, &root, &path)?;
            let name = path
                .strip_prefix(&root)
                .map_err(|_| "document escapes root")?
                .to_str()
                .ok_or("non-UTF8 document")?
                .replace('\\', "/");
            if !super::configuration::document_output(&name) {
                return Err("not a consumer document output".into());
            }
            if !matches!(inspect(&root, &path, false)?, State::File(_)) {
                return Err("document destination is not a file".into());
            }
            writes.insert(path, text.into_bytes());
        }
        let mut removes = Vec::new();
        if let Some(skills) = skills {
            for (path, bytes) in skills.writes {
                let path = derived_path(original, &root, &path)?;
                writes.insert(path, bytes);
            }
            for path in skills.removes {
                removes.push(derived_path(original, &root, &path)?);
            }
        }
        let mut observed = BTreeMap::new();
        // Preflight every leaf AND parent kind before any document effect.
        for path in writes.keys().chain(removes.iter()) {
            let state = inspect(&root, path, true)?;
            if matches!(state, State::Directory) {
                return Err("output file is a directory".into());
            }
            if removes.contains(path) && !matches!(state, State::File(_)) {
                return Err("stale derived file changed kind".into());
            }
            observed.insert(path.clone(), state);
            let mut parent = path.parent();
            while let Some(directory) = parent.filter(|p| *p != root) {
                let state = inspect(&root, directory, true)?;
                if matches!(state, State::File(_)) {
                    return Err("output directory is a file".into());
                }
                observed.insert(directory.to_owned(), state);
                parent = directory.parent();
            }
        }
        for path in writes.keys() {
            let temporary = temporary(path)?;
            if !matches!(inspect(&root, &temporary, true)?, State::Missing) {
                return Err("guidance temporary destination already exists".into());
            }
            observed.insert(temporary, State::Missing);
        }
        Ok(Self {
            root,
            observed,
            writes,
            removes,
        })
    }
    pub(crate) fn apply(self) -> Result<(), String> {
        // Revalidate the entire planned operation set before the first effect.
        for (path, before) in &self.observed {
            if &inspect(&self.root, path, true)? != before {
                return Err(format!("guidance destination changed: {}", path.display()));
            }
        }
        for (path, bytes) in self.writes {
            let parent = path.parent().ok_or("missing output parent")?;
            if self.observed.get(&path) != Some(&inspect(&self.root, &path, true)?) {
                return Err("guidance file changed before application".into());
            }
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            inspect(&self.root, &path, true)?;
            let temporary = temporary(&path)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|e| e.to_string())?;
            let result = (|| {
                file.write_all(&bytes).map_err(|e| e.to_string())?;
                file.sync_all().map_err(|e| e.to_string())?;
                drop(file);
                if self.observed.get(&path) != Some(&inspect(&self.root, &path, true)?) {
                    return Err("guidance file changed before publication".into());
                }
                std::fs::rename(&temporary, &path).map_err(|e| e.to_string())
            })();
            if result.is_err() {
                let _ = std::fs::remove_file(&temporary);
            }
            result?;
        }
        for path in self.removes {
            if self.observed.get(&path) != Some(&inspect(&self.root, &path, false)?) {
                return Err("stale output is no longer a regular file".into());
            }
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn temporary(path: &Path) -> Result<PathBuf, String> {
    Ok(path
        .parent()
        .ok_or("missing output parent")?
        .join(format!(".worth-guidance-{}.tmp", std::process::id())))
}

fn normalize(original: &Path, canonical: &Path, path: &Path) -> Result<PathBuf, String> {
    let relative = path
        .strip_prefix(canonical)
        .or_else(|_| path.strip_prefix(original))
        .map_err(|_| "output is outside the selected checkout")?;
    Ok(canonical.join(relative))
}
fn derived_path(original: &Path, root: &Path, path: &Path) -> Result<PathBuf, String> {
    let path = normalize(original, root, path)?;
    let zone = root.join(".grok/skills");
    let name = path
        .strip_prefix(&zone)
        .map_err(|_| "not a derived skill output")?;
    if name.as_os_str().is_empty() {
        return Err("empty derived path".into());
    }
    inspect(root, &path, true)?;
    Ok(path)
}
