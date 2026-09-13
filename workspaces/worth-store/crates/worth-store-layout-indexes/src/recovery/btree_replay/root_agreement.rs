use worth_store_physical_format::{
    CheckpointWalSourceRange, PhysicalCheckpointIdentity, PhysicalReference,
};
use worth_store_recovery_physics::PhysicalSourceSelection;
use worth_store_wal::WalLsnRange;

use super::BTreeReplaySourceDenial;

/// Recovery-owned proof that one checkpoint/WAL source names the exact B-tree root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BTreeReplayRootAgreement {
    checkpoint_id: PhysicalCheckpointIdentity,
    root_reference: PhysicalReference,
    checkpoint_coverage: CheckpointWalSourceRange,
    replay_tail: WalLsnRange,
}

impl BTreeReplayRootAgreement {
    pub(super) fn admit(
        source: &PhysicalSourceSelection,
        root_reference: PhysicalReference,
    ) -> Result<Self, BTreeReplaySourceDenial> {
        let checkpoint = source
            .checkpoint()
            .ok_or(BTreeReplaySourceDenial::NoAdmittedDurableSource)?;
        let first_segment = source
            .wal_tail()
            .segments()
            .first()
            .ok_or(BTreeReplaySourceDenial::WalOnlyRootNotMaterialized)?;
        let last_segment = source
            .wal_tail()
            .segments()
            .last()
            .ok_or(BTreeReplaySourceDenial::WalOnlyRootNotMaterialized)?;
        let checkpoint_coverage = checkpoint.checkpoint().source().wal();
        let replay_tail = WalLsnRange::new(
            first_segment.inspection().lsn_range().start(),
            last_segment.inspection().lsn_range().end_exclusive(),
        )
        .map_err(|_| BTreeReplaySourceDenial::CheckpointTailIdentityMismatch)?;
        if checkpoint_coverage.covered_end_lsn_exclusive() != replay_tail.start().get() {
            return Err(BTreeReplaySourceDenial::CheckpointTailFrontierMismatch {
                checkpoint_end: checkpoint_coverage.covered_end_lsn_exclusive(),
                tail_start: replay_tail.start().get(),
            });
        }

        Ok(Self {
            checkpoint_id: checkpoint.checkpoint().source().identity(),
            root_reference,
            checkpoint_coverage,
            replay_tail,
        })
    }

    pub fn checkpoint_id(&self) -> &PhysicalCheckpointIdentity {
        &self.checkpoint_id
    }

    pub const fn root_reference(&self) -> PhysicalReference {
        self.root_reference
    }

    pub const fn checkpoint_coverage(&self) -> CheckpointWalSourceRange {
        self.checkpoint_coverage
    }

    pub const fn replay_tail(&self) -> WalLsnRange {
        self.replay_tail
    }
}
