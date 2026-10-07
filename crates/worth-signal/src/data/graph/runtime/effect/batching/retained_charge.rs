//! Exact retained heap measurement for worker-local apply packets.
use crate::data::dependency::CommittedSnapshotUpdate;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
use crate::logic::evaluation::{
    EffectRuntimeMetadata, EvaluationEffect, OperationalEffect, PendingDependencySnapshot,
    PreviousArtifactWarmSnapshot,
};

use super::{ApplyCommitPacket, PreparedParallelApplyCommitPacket};

impl worth_execution::ChargedBytes for PreparedParallelApplyCommitPacket {
    fn additional_charged_bytes(&self) -> u64 {
        self.0
            .retained_heap_charge(&mut Preparation::new(usize::MAX))
            .map_or(u64::MAX, Charge::bytes)
    }
}

impl RetainedStorageMeasurement for ApplyCommitPacket {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.effect
            .retained_heap_charge(work)?
            .checked_add(self.pending_snapshot.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for EvaluationEffect {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.operational
            .retained_heap_charge(work)?
            .checked_add(self.diagnostics.retained_heap_charge(work)?)?
            .checked_add(self.runtime_metadata.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for OperationalEffect {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.changed_aspect_regions
            .retained_heap_charge(work)?
            .checked_add(self.reuse_basis.retained_heap_charge(work)?)?
            .checked_add(self.reuse_boundary_authority.retained_heap_charge(work)?)?
            .checked_add(self.dependency_snapshot_update.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for EffectRuntimeMetadata {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.keyed_context
            .retained_heap_charge(work)?
            .checked_add(self.causality.retained_heap_charge(work)?)?
            .checked_add(self.reuse_certification.retained_heap_charge(work)?)?
            .checked_add(self.reuse_boundary_detail.retained_heap_charge(work)?)?
            .checked_add(self.previous_artifact_warm.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for PreviousArtifactWarmSnapshot {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.output_identity
            .retained_heap_charge(work)?
            .checked_add(self.continuity_token.retained_heap_charge(work)?)?
            .checked_add(self.reuse_boundary_authority.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for PendingDependencySnapshot {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.update.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for CommittedSnapshotUpdate {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::VersionOnly(update) => update.versions().retained_heap_charge(work),
            Self::Replace(update) => update.snapshot().snapshot().retained_heap_charge(work),
        }
    }
}
