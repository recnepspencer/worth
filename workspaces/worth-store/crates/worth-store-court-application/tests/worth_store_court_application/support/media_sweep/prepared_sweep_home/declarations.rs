//! Worker-local saved homes with exact differential restoration.
//!
//! Store's durability admission basis hashes the root directory's filesystem
//! identity (`volume_serial` + `file_index` on Windows, `dev` + `ino` on Unix)
//! into `policy_identity`, so every worker restores bytes under its own
//! original root directory. Differential restoration deletes only extra/type-
//! mismatched entries, recreates missing directories, and rewrites only files
//! whose on-disk length or bytes differ from the cached immutable image.

pub(crate) use std::collections::BTreeMap;
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::sync::Arc;

pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use worth_relational::facade::schema::SchemaVersionId;

pub(crate) use {crate::durable_run, crate::prepared_home::PreparedHome, crate::workload::Step};

/// Immutable pristine file bytes and permission state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PristineFile {
    pub(crate) bytes: Arc<[u8]>,
    pub(crate) readonly: bool,
}

/// Statistics for one differential restoration pass (used by diagnostics & self-tests).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RestoreStats {
    pub(crate) removed_entries: usize,
    pub(crate) created_dirs: usize,
    pub(crate) rewritten_files: usize,
    pub(crate) unchanged_files: usize,
    pub(crate) metadata_fixed: usize,
}

/// One worker's closed prefix and immutable file image. No live Store or
/// fault activation survives a case. Each worker owns a separate root/image;
/// restoring brings the root back byte-for-byte without changing root identity.
pub(crate) struct PreparedSweepHome {
    pub(crate) home: PreparedHome,
    pub(crate) schema: SchemaVersionId,
    pub(crate) image: Arc<DirectoryImage>,
}

impl PreparedSweepHome {
    pub(crate) fn prepare(steps: &[Step], execution: &ExecutionRequest<'_, '_>) -> Self {
        let home = crate::prepared_home::PreparedHome::prepare(steps, execution);
        let image = Arc::new(DirectoryImage::read(home.location()));
        Self {
            home,
            schema: crate::durable_run::served_schema(steps),
            image,
        }
    }

    pub(crate) fn location(&self) -> &Path {
        self.home.location()
    }

    pub(crate) fn schema(&self) -> SchemaVersionId {
        self.schema
    }

    /// Differentially restore `self.location()` to exact byte-for-byte and
    /// permission parity with `self.image`.
    pub(crate) fn restore(&mut self) {
        let _ = self.image.restore_differential(self.location());
    }
}

/// Preserve every file's bytes, every directory (including empty ones), and
/// read-only flags. Reject links and special files rather than sharing mutable
/// backing state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DirectoryImage {
    /// Relative directory paths (including `""` for root) mapped to `readonly`.
    pub(crate) directories: BTreeMap<PathBuf, bool>,
    /// Relative file paths mapped to immutable pristine bytes & `readonly`.
    pub(crate) files: BTreeMap<PathBuf, PristineFile>,
}

impl DirectoryImage {
    pub(crate) fn read(root: &Path) -> Self {
        let root_readonly = std::fs::symlink_metadata(root)
            .expect("read root metadata")
            .permissions()
            .readonly();
        let mut directories = BTreeMap::new();
        directories.insert(PathBuf::new(), root_readonly);
        let mut files = BTreeMap::new();
        let mut queue = vec![PathBuf::new()];
        let mut next = 0;
        while let Some(relative) = queue.get(next).cloned() {
            let mut entries: Vec<_> = std::fs::read_dir(root.join(&relative))
                .expect("read closed prefix")
                .map(|entry| entry.expect("closed prefix entry"))
                .collect();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                let path = relative.join(entry.file_name());
                let meta = std::fs::symlink_metadata(entry.path()).expect("closed prefix metadata");
                let kind = meta.file_type();
                assert!(
                    !kind.is_symlink(),
                    "a home image must not contain symlinks: {}",
                    path.display()
                );
                let readonly = meta.permissions().readonly();
                if kind.is_dir() {
                    directories.insert(path.clone(), readonly);
                    queue.push(path);
                } else {
                    assert!(
                        kind.is_file(),
                        "a home image contains only files/directories: {}",
                        path.display()
                    );
                    let bytes: Arc<[u8]> = std::fs::read(entry.path())
                        .expect("read closed prefix file")
                        .into();
                    files.insert(path, PristineFile { bytes, readonly });
                }
            }
            next += 1;
        }
        Self { directories, files }
    }

    pub(crate) fn write(&self, root: &Path) {
        for path in self.directories.keys() {
            if !path.as_os_str().is_empty() {
                std::fs::create_dir_all(root.join(path)).expect("restore prefix directory");
            }
        }
        for (path, pristine) in &self.files {
            let full = root.join(path);
            std::fs::write(&full, pristine.bytes.as_ref()).expect("restore prefix bytes");
            if pristine.readonly {
                set_readonly(&full, true);
            }
        }
        apply_readonly_dirs(root, &self.directories);
    }

    /// Restore `root` in-place to match `self` byte-for-byte and permission-for-
    /// permission, touching only entries that were added, deleted, type-swapped,
    /// or modified. Refuses to traverse or mutate through a symlink root.
    pub(crate) fn restore_differential(&self, root: &Path) -> RestoreStats {
        let root_meta = std::fs::symlink_metadata(root).expect("stat restore root");
        assert!(
            !root_meta.file_type().is_symlink() && root_meta.file_type().is_dir(),
            "restore root must be a real directory, not a symlink: {}",
            root.display()
        );
        let mut stats = RestoreStats::default();
        ensure_writable(root);

        // Phase 1: remove unexpected files, unexpected directories, symlinks,
        // and type-mismatched entries without following symlinks.
        self.prune_unexpected(root, Path::new(""), &mut stats);

        // Phase 2: ensure every pristine directory exists and is writable while
        // restoring child files.
        for relative in self.directories.keys() {
            if relative.as_os_str().is_empty() {
                continue;
            }
            let full = root.join(relative);
            match std::fs::symlink_metadata(&full) {
                Ok(meta) if !meta.file_type().is_symlink() && meta.file_type().is_dir() => {
                    if meta.permissions().readonly() {
                        set_readonly(&full, false);
                    }
                }
                Ok(_) => {
                    remove_entry_writable(&full);
                    std::fs::create_dir(&full).expect("recreate pristine directory");
                    stats.created_dirs += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    std::fs::create_dir(&full).expect("create missing pristine directory");
                    stats.created_dirs += 1;
                }
                Err(error) => panic!("stat pristine directory {}: {error}", full.display()),
            }
        }

        // Phase 3: restore every pristine file, rewriting only when missing or
        // bytes/length differ.
        for (relative, pristine) in &self.files {
            let full = root.join(relative);
            let mut needs_write = true;
            let mut current_readonly = false;
            if let Ok(meta) = std::fs::symlink_metadata(&full) {
                let kind = meta.file_type();
                if !kind.is_symlink() && kind.is_file() {
                    current_readonly = meta.permissions().readonly();
                    if meta.len() == pristine.bytes.len() as u64 {
                        let existing =
                            std::fs::read(&full).expect("read existing file for diff restore");
                        if existing.as_slice() == pristine.bytes.as_ref() {
                            needs_write = false;
                        }
                    }
                } else {
                    remove_entry_writable(&full);
                    stats.removed_entries += 1;
                }
            }
            if needs_write {
                if current_readonly {
                    set_readonly(&full, false);
                }
                std::fs::write(&full, pristine.bytes.as_ref()).expect("rewrite modified file");
                if pristine.readonly {
                    set_readonly(&full, true);
                }
                stats.rewritten_files += 1;
            } else {
                if current_readonly != pristine.readonly {
                    set_readonly(&full, pristine.readonly);
                    stats.metadata_fixed += 1;
                }
                stats.unchanged_files += 1;
            }
        }

        // Phase 4: restore read-only flags on directories (deepest-first).
        apply_readonly_dirs(root, &self.directories);
        stats
    }

    fn prune_unexpected(&self, root: &Path, relative: &Path, stats: &mut RestoreStats) {
        let dir_path = root.join(relative);
        ensure_writable(&dir_path);
        for entry in std::fs::read_dir(&dir_path).expect("read directory for differential restore")
        {
            let entry = entry.expect("differential restore dir entry");
            let child_rel = relative.join(entry.file_name());
            let child_full = entry.path();
            let meta = std::fs::symlink_metadata(&child_full)
                .expect("differential restore entry metadata");
            let kind = meta.file_type();
            if kind.is_symlink() {
                remove_symlink(&child_full, kind);
                stats.removed_entries += 1;
            } else if kind.is_dir() {
                if self.directories.contains_key(&child_rel) {
                    self.prune_unexpected(root, &child_rel, stats);
                } else {
                    remove_dir_all_writable(&child_full);
                    stats.removed_entries += 1;
                }
            } else if !kind.is_file() || !self.files.contains_key(&child_rel) {
                if meta.permissions().readonly() {
                    set_readonly(&child_full, false);
                }
                std::fs::remove_file(&child_full).expect("remove unexpected case file");
                stats.removed_entries += 1;
            }
        }
    }
}

fn ensure_writable(path: &Path) {
    if let Ok(meta) = std::fs::symlink_metadata(path)
        && !meta.file_type().is_symlink()
        && meta.permissions().readonly()
    {
        set_readonly(path, false);
    }
}

fn set_readonly(path: &Path, readonly: bool) {
    let meta = std::fs::symlink_metadata(path).expect("read permissions");
    assert!(
        !meta.file_type().is_symlink(),
        "must not modify permissions through symlink: {}",
        path.display()
    );
    let mut perms = meta.permissions();
    perms.set_readonly(readonly);
    std::fs::set_permissions(path, perms).expect("set permissions");
}

fn remove_symlink(path: &Path, kind: std::fs::FileType) {
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileTypeExt;
        if kind.is_symlink_dir() {
            std::fs::remove_dir(path).expect("remove directory symlink");
            return;
        }
    }
    #[cfg(not(windows))]
    let _ = kind;
    if std::fs::remove_file(path).is_err() {
        std::fs::remove_dir(path).expect("remove symlink entry");
    }
}

fn remove_entry_writable(path: &Path) {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    let kind = meta.file_type();
    if kind.is_symlink() {
        remove_symlink(path, kind);
    } else if kind.is_dir() {
        remove_dir_all_writable(path);
    } else {
        if meta.permissions().readonly() {
            set_readonly(path, false);
        }
        std::fs::remove_file(path).expect("remove file entry");
    }
}

fn remove_dir_all_writable(path: &Path) {
    ensure_writable(path);
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
            let entry = entry.expect("read dir entry for removal");
            remove_entry_writable(&entry.path());
        }
    }
    std::fs::remove_dir(path).expect("remove directory");
}

fn apply_readonly_dirs(root: &Path, directories: &BTreeMap<PathBuf, bool>) {
    let mut ordered: Vec<(&PathBuf, bool)> = directories
        .iter()
        .filter(|(_, readonly)| **readonly)
        .map(|(path, readonly)| (path, *readonly))
        .collect();
    ordered.sort_by(|a, b| b.0.components().count().cmp(&a.0.components().count()));
    for (relative, readonly) in ordered {
        let full = if relative.as_os_str().is_empty() {
            root.to_path_buf()
        } else {
            root.join(relative)
        };
        set_readonly(&full, readonly);
    }
}
