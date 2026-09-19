use super::{
    ArtifactFamilyId, PersistentCorrespondenceEvidence, ReuseSemanticRegionIdentity,
    ReuseStrategyBoundaryContext,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ArtifactFamilyId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ReuseSemanticRegionIdentity {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            partitioned_output: _,
            partition_scope,
            required_context: _,
        } = self;
        partition_scope.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for PersistentCorrespondenceEvidence {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::HostSuppliedKey(value)
            | Self::ContractDeclaredBasis(value)
            | Self::LineageBackedMapping(value)
            | Self::RegionIdentityBasis(value) => value.retained_heap_charge(work),
        }
    }
}

impl RetainedStorageMeasurement for ReuseStrategyBoundaryContext {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::None => Ok(Charge::ZERO),
            Self::CrossIdentity {
                persistent_correspondence,
            } => persistent_correspondence.retained_heap_charge(work),
            Self::PartialArtifactSplice {
                composition_regions,
            } => composition_regions.retained_heap_charge(work),
        }
    }
}
