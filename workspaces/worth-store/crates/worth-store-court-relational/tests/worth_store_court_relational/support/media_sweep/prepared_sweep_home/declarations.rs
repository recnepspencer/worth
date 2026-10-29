//! Worker-local saved homes. Restore bytes under the original root: before
//! the first checkpoint Store's WAL policy binds that root's filesystem ID,
//! so copying a prepared home to another directory cannot recover it.

pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::schema::SchemaVersionId};

pub(crate) use {crate::durable_run, crate::prepared_home::PreparedHome, crate::workload::Step};

/// One worker's closed prefix and immutable file image. No live Store or
/// fault activation survives a case. Each worker owns a separate root/image;
/// restoring deletes all case output without changing the root identity.
pub(crate) struct PreparedSweepHome {
    pub(crate) home: PreparedHome,
    pub(crate) schema: SchemaVersionId,
    pub(crate) image: DirectoryImage,
}

impl PreparedSweepHome {
    pub(crate) fn prepare(steps: &[Step], execution: &ExecutionRequest<'_, '_>) -> Self {
        let home = crate::prepared_home::PreparedHome::prepare(steps, execution);
        let image = crate::media_sweep::prepared_sweep_home::DirectoryImage::read(home.location());
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

    /// O(saved bytes + current entries), with no script execution, flush
    /// barriers or admission changes. The next case reopens through Store.
    pub(crate) fn restore(&mut self) {
        for entry in std::fs::read_dir(self.location()).expect("read case output") {
            let entry = entry.expect("case output entry");
            if entry.file_type().expect("case output type").is_dir() {
                std::fs::remove_dir_all(entry.path()).expect("remove case output directory");
            } else {
                std::fs::remove_file(entry.path()).expect("remove case output file");
            }
        }
        self.image.write(self.location());
    }
}

/// Preserve every file's bytes and every directory (including empty ones).
/// Reject links and special files rather than sharing mutable backing state.
pub(crate) struct DirectoryImage {
    pub(crate) directories: Vec<PathBuf>,
    pub(crate) files: Vec<(PathBuf, Vec<u8>)>,
}

impl DirectoryImage {
    pub(crate) fn read(root: &Path) -> Self {
        let mut image = Self {
            directories: vec![std::path::PathBuf::new()],
            files: Vec::new(),
        };
        let mut next = 0;
        while let Some(relative) = image.directories.get(next).cloned() {
            for entry in std::fs::read_dir(root.join(&relative)).expect("read closed prefix") {
                let entry = entry.expect("closed prefix entry");
                let path = relative.join(entry.file_name());
                let kind = entry.file_type().expect("closed prefix type");
                if kind.is_dir() {
                    image.directories.push(path);
                } else {
                    assert!(
                        kind.is_file(),
                        "a home image contains only files/directories"
                    );
                    image.files.push((
                        path,
                        std::fs::read(entry.path()).expect("read closed prefix file"),
                    ));
                }
            }
            next += 1;
        }
        image
    }

    pub(crate) fn write(&self, root: &Path) {
        for path in self.directories.iter().skip(1) {
            std::fs::create_dir(root.join(path)).expect("restore prefix directory");
        }
        for (path, bytes) in &self.files {
            std::fs::write(root.join(path), bytes).expect("restore prefix bytes");
        }
    }
}