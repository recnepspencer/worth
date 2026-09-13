use super::{
    BranchConflictResolutionPlan, BranchMergeConflictKind, BranchMergeResolutionRequirement,
    ConflictResolutionRecord, ConflictResolutionStrategy,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for BranchConflictResolutionPlan {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source_branch_id: _,
            target_branch_id: _,
            divergence: _,
            records,
        } = self;
        records.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for ConflictResolutionRecord {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source_node: _,
            target_node: _,
            required_resolution,
            supported_strategies,
        } = self;
        required_resolution
            .retained_heap_charge(work)?
            .checked_add(supported_strategies.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for BranchMergeConflictKind {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::ComparableMismatch
            | Self::DependencyTopologyMismatch
            | Self::DependencySnapshotMismatch
            | Self::RuntimeArtifactMismatch
            | Self::MergeAuthorityMismatch => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for BranchMergeResolutionRequirement {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::ReconcileComparableState
            | Self::ReconcileDependencyTopology
            | Self::ReconcileDependencySnapshot
            | Self::ReconcileRuntimeArtifactState
            | Self::ReconcileMergeAuthority => Ok(Charge::ZERO),
        }
    }
}
impl RetainedStorageMeasurement for ConflictResolutionStrategy {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::AdoptSourceComparableState
            | Self::PreserveTargetComparableState
            | Self::ReplaySourceDependencyTopology
            | Self::PreserveTargetDependencyTopology
            | Self::ReplaySourceDependencySnapshot
            | Self::PreserveTargetDependencySnapshot
            | Self::AdoptSourceRuntimeArtifactState
            | Self::PreserveTargetRuntimeArtifactState
            | Self::AdoptSourceMergeAuthority
            | Self::PreserveTargetMergeAuthority => Ok(Charge::ZERO),
        }
    }
}
