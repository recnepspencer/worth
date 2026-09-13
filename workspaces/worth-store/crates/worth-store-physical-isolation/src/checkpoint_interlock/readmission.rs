use super::CheckpointReadInterlockDenial;
use crate::{CheckpointPublicationRoot, CurrentPhysicalRoot};
use worth_store_physical_format::{CheckpointWalSourceRange, PhysicalCheckpointSource};
use worth_store_physical_integrity::{
    VerifiedCheckpointCompactionCutover, VerifiedCheckpointStream,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointPublicationReadmission {
    checkpoint_root: CheckpointPublicationRoot,
    published_current_root: CurrentPhysicalRoot,
    checkpoint_source: PhysicalCheckpointSource,
    compaction_cutover: VerifiedCheckpointCompactionCutover,
    checkpoint_wal_bound_to_cutover: bool,
}

impl CheckpointPublicationReadmission {
    pub fn admit(
        checkpoint_root: CheckpointPublicationRoot,
        published_current_root: CurrentPhysicalRoot,
        checkpoint: &VerifiedCheckpointStream,
    ) -> Result<Self, CheckpointReadInterlockDenial> {
        let checkpoint_source = checkpoint.source();
        let compaction_cutover = checkpoint.compaction_cutover();
        if checkpoint_root.epoch() != published_current_root.epoch() {
            return Err(
                CheckpointReadInterlockDenial::CheckpointPublicationRootNotReadmitted {
                    checkpoint_root: checkpoint_root.epoch(),
                    admitted_root: published_current_root.epoch(),
                },
            );
        }
        if !checkpoint_root
            .checkpoint_identity()
            .matches_physical_checkpoint_identity(checkpoint_source.identity())
        {
            return Err(CheckpointReadInterlockDenial::CheckpointPublicationRootCheckpointMismatch);
        }
        if compaction_cutover.checkpoint() != checkpoint_source.identity()
            || compaction_cutover.root() != checkpoint_source.root()
        {
            return Err(
                CheckpointReadInterlockDenial::CheckpointPublicationCompactionProductMismatch,
            );
        }
        let checkpoint_range = checkpoint_source.wal();
        let product_range = compaction_cutover.checkpoint_wal();
        if product_range != checkpoint_range {
            return Err(
                CheckpointReadInterlockDenial::CheckpointPublicationWalRangeMismatch {
                    checkpoint_range,
                    product_range,
                },
            );
        }
        let cutoff_lsn = compaction_cutover.wal_cutoff_lsn_exclusive();
        if cutoff_lsn < checkpoint_range.admitted_begin_lsn()
            || cutoff_lsn > checkpoint_range.covered_end_lsn_exclusive()
        {
            return Err(
                CheckpointReadInterlockDenial::CompactionCutoffOutsideCheckpointWalRange {
                    cutoff_lsn,
                    checkpoint_range,
                },
            );
        }
        Ok(Self {
            checkpoint_root,
            published_current_root,
            checkpoint_source,
            compaction_cutover,
            checkpoint_wal_bound_to_cutover: true,
        })
    }

    pub const fn checkpoint_root(&self) -> &CheckpointPublicationRoot {
        &self.checkpoint_root
    }

    pub const fn published_current_root(&self) -> CurrentPhysicalRoot {
        self.published_current_root
    }

    pub const fn checkpoint_source(&self) -> PhysicalCheckpointSource {
        self.checkpoint_source
    }

    pub const fn compaction_cutover(&self) -> VerifiedCheckpointCompactionCutover {
        self.compaction_cutover
    }

    pub const fn checkpoint_wal_range(&self) -> CheckpointWalSourceRange {
        self.checkpoint_source.wal()
    }

    pub const fn checkpoint_wal_bound_to_cutover(&self) -> bool {
        self.checkpoint_wal_bound_to_cutover
    }
}
