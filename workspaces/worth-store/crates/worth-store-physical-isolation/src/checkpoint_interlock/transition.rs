use super::{
    CheckpointPublicationReadmission, CheckpointReadInterlockCounters,
    CheckpointReadInterlockDenial,
};
use crate::{CheckpointPublicationRoot, CurrentPhysicalRoot};
use worth_store_physical_format::{CheckpointWalSourceRange, PhysicalCheckpointSource};
use worth_store_physical_integrity::VerifiedCheckpointCompactionCutover;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointRootEpochTransition {
    old_current_root: CurrentPhysicalRoot,
    readmission: CheckpointPublicationReadmission,
    counters: CheckpointReadInterlockCounters,
}

impl CheckpointRootEpochTransition {
    pub fn admit(
        old_current_root: CurrentPhysicalRoot,
        readmission: CheckpointPublicationReadmission,
    ) -> Result<Self, CheckpointReadInterlockDenial> {
        let published_current_root = readmission.published_current_root();
        if published_current_root.epoch() == old_current_root.epoch() {
            return Err(CheckpointReadInterlockDenial::StaleCheckpointRootEpoch {
                old_root: old_current_root.epoch(),
                published_root: published_current_root.epoch(),
            });
        }
        if published_current_root.manifest_epoch() == old_current_root.manifest_epoch() {
            return Err(
                CheckpointReadInterlockDenial::StaleCheckpointManifestEpoch {
                    old_manifest: old_current_root.manifest_epoch(),
                    published_manifest: published_current_root.manifest_epoch(),
                },
            );
        }
        let counters = CheckpointReadInterlockCounters::admitted();
        Ok(Self {
            old_current_root,
            readmission,
            counters,
        })
    }

    pub const fn old_current_root(&self) -> CurrentPhysicalRoot {
        self.old_current_root
    }

    pub const fn checkpoint_root(&self) -> &CheckpointPublicationRoot {
        self.readmission.checkpoint_root()
    }

    pub const fn published_current_root(&self) -> CurrentPhysicalRoot {
        self.readmission.published_current_root()
    }

    pub const fn checkpoint_source(&self) -> PhysicalCheckpointSource {
        self.readmission.checkpoint_source()
    }

    pub const fn compaction_cutover(&self) -> VerifiedCheckpointCompactionCutover {
        self.readmission.compaction_cutover()
    }

    pub const fn checkpoint_wal_range(&self) -> CheckpointWalSourceRange {
        self.readmission.checkpoint_wal_range()
    }

    pub const fn checkpoint_wal_bound_to_cutover(&self) -> bool {
        self.readmission.checkpoint_wal_bound_to_cutover()
    }

    pub const fn counters(&self) -> CheckpointReadInterlockCounters {
        self.counters
    }
}
