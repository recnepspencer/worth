use super::{ReuseBasis, ReuseOrigin};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ReuseBasis {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            strategy: _,
            source: _,
            crossing: _,
            dependency_snapshot_basis: _,
            topology_regime_basis: _,
            structural_dependency_basis: _,
            artifact_family_basis,
            partition_region_basis_count: _,
        } = self;
        artifact_family_basis.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ReuseOrigin {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::FreshCompute
            | Self::OutputSuppressed
            | Self::MemoizedArtifactReuse
            | Self::SnapshotRestore
            | Self::ReconciliationAdoption
            | Self::CrossIdentityPersistentReuse
            | Self::PartialArtifactSplice => Ok(Charge::ZERO),
        }
    }
}
