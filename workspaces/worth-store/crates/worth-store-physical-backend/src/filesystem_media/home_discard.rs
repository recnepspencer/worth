//! Destructive namespace disposal under the ordinary Store mutation lease.
mod removal;
use super::{
    AdmittedStoreNamespace, MediaFaultSchedule, MutationOwnershipDenial, MutationOwnershipLease,
};
use cap_fs_ext::DirExt;
use cap_std::{ambient_authority, fs::Dir};
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Presence, including a torn frame, fences ordinary opening until disposal finishes.
pub const HOME_DISCARD_TOMBSTONE: &str = "discard.pending";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemHomeDiscardStage {
    Admission,
    Tombstone,
    Removal,
    Completion,
}

#[derive(Debug)]
pub enum FilesystemHomeDiscardDenial {
    MutationOwnerContended,
    Ownership(MutationOwnershipDenial),
    Confinement(super::NamespaceConfinementDenial),
    Io {
        stage: FilesystemHomeDiscardStage,
        kind: io::ErrorKind,
        raw_os_error: Option<i32>,
    },
}
impl FilesystemHomeDiscardDenial {
    fn io(stage: FilesystemHomeDiscardStage, error: io::Error) -> Self {
        Self::Io {
            stage,
            kind: error.kind(),
            raw_os_error: error.raw_os_error(),
        }
    }
}

/// Does not initialize, recover or read any Store artifact. The existing lock
/// inode and its parent are retained, so a competing opener cannot lock a replacement.
pub struct FilesystemHomeDiscard {
    root: PathBuf,
    schedule: MediaFaultSchedule,
}
impl FilesystemHomeDiscard {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            schedule: MediaFaultSchedule::default(),
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn with_fault_schedule(mut self, schedule: MediaFaultSchedule) -> Self {
        self.schedule = schedule;
        self
    }

    pub fn execute(self, intent: &[u8]) -> Result<(), FilesystemHomeDiscardDenial> {
        use FilesystemHomeDiscardDenial as Denial;
        use FilesystemHomeDiscardStage as Stage;
        let counters = Arc::new(super::operation_counters::MediaCounterCells::default());
        let boundary =
            super::fault_interposition::MediaFaultInterposer::new(self.schedule, counters);
        let (parent, root) = open_root(&self.root).map_err(|e| Denial::io(Stage::Admission, e))?;
        create_directory(&root, "namespace").map_err(|e| Denial::io(Stage::Admission, e))?;
        let mut namespace = AdmittedStoreNamespace::from_opened_directory(root, &boundary)
            .map_err(Denial::Confinement)?;
        let directory = namespace.open_role_directory(
            worth_store_physical_format::store_namespace::StoreNamespaceRelativeRole::NamespaceDirectory,
            &boundary,
        ).map_err(Denial::Confinement)?;
        let lease = MutationOwnershipLease::try_acquire_or_create(
            namespace.owner_identity(),
            &directory,
            &boundary,
        )
        .map_err(|failure| match failure.denial() {
            MutationOwnershipDenial::Contended => Denial::MutationOwnerContended,
            cause => Denial::Ownership(cause),
        })?;
        // All handles and the actual lease stay alive through the final barrier.
        let result = removal::discard(
            namespace.root_handle().directory(),
            directory.directory(),
            &parent,
            intent,
            &boundary,
        );
        let release = lease.release(&boundary);
        result?;
        match release {
            super::OwnershipReleaseOutcome::Released => Ok(()),
            super::OwnershipReleaseOutcome::ReleaseUnconfirmed { kind, raw_os_error } => {
                Err(Denial::Io {
                    stage: Stage::Completion,
                    kind,
                    raw_os_error,
                })
            }
        }
    }
}

fn open_root(path: &Path) -> io::Result<(Dir, Dir)> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = Dir::open_ambient_dir(parent, ambient_authority())?;
    create_directory(&parent, name)?;
    let root = parent.open_dir_nofollow(name)?;
    Ok((parent, root))
}

fn create_directory(parent: &Dir, name: impl AsRef<Path>) -> io::Result<()> {
    match parent.create_dir(name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}
