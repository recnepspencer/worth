//! Dispose of a filesystem Home without opening Store, including damaged data.
use std::path::{Path, PathBuf};
use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolVersion, DurableArtifactRetention, RetentionClass,
};
use worth_store::physical_runtime::{FilesystemHomeDiscard, FilesystemHomeDiscardDenial};

pub use worth_store::physical_runtime::FilesystemHomeDiscardStage as ApplicationHomeDiscardStage;

#[derive(Debug)]
pub enum ApplicationHomeDiscardDenial {
    MutationOwnerContended,
    Filesystem(FilesystemHomeDiscardDenial),
}

/// This operation destroys all contents of the named Home except its live-lock
/// scaffold. Successful completion permits a new Store identity and incarnation.
pub fn discard_home(location: impl AsRef<Path>) -> Result<(), ApplicationHomeDiscardDenial> {
    ApplicationHomeDiscard::new(location.as_ref().to_owned()).execute()
}

/// One destructive request; interruption is completed by another request for
/// the same directory. Certification faults are placed on this receiver only.
pub struct ApplicationHomeDiscard {
    filesystem: FilesystemHomeDiscard,
}
impl ApplicationHomeDiscard {
    pub fn new(location: PathBuf) -> Self {
        Self {
            filesystem: FilesystemHomeDiscard::new(location),
        }
    }

    pub fn execute(self) -> Result<(), ApplicationHomeDiscardDenial> {
        self.filesystem
            .execute(&HomeDiscardIntent::encode())
            .map_err(|cause| match cause {
                FilesystemHomeDiscardDenial::MutationOwnerContended => {
                    ApplicationHomeDiscardDenial::MutationOwnerContended
                }
                cause @ (FilesystemHomeDiscardDenial::Ownership(_)
                | FilesystemHomeDiscardDenial::Confinement(_)
                | FilesystemHomeDiscardDenial::Io { .. }) => {
                    ApplicationHomeDiscardDenial::Filesystem(cause)
                }
            })
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn with_fault_schedule(
        mut self,
        schedule: worth_store::physical_runtime::certification::MediaFaultSchedule,
    ) -> Self {
        self.filesystem = self.filesystem.with_fault_schedule(schedule);
        self
    }
}

struct HomeDiscardIntent;
impl DurableArtifactRetention for HomeDiscardIntent {
    const RETENTION_CLASS: RetentionClass = RetentionClass::MandatoryObligation;
}
impl HomeDiscardIntent {
    fn encode() -> Vec<u8> {
        // Presence is the fence: torn/unknown frames also require explicit disposal.
        const PROTOCOL: BoundaryProtocolIdentity =
            BoundaryProtocolIdentity::new("worth.application.home.discard");
        const VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
        let mut bytes = Vec::from(PROTOCOL.as_str().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&VERSION.get().to_le_bytes());
        bytes
    }
}

pub(crate) fn pending(location: &Path) -> std::io::Result<bool> {
    match std::fs::symlink_metadata(
        location.join(worth_store::physical_runtime::HOME_DISCARD_TOMBSTONE),
    ) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
