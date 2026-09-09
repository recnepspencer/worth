use super::{ReuseBoundaryAuthority, ReuseStrategyBoundaryAuthority};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ReuseBoundaryAuthority {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            topology_regime: _,
            tolerance_regime,
            semantic_region_digest: _,
            authority_policy: _,
            artifact_family,
            structural_dependency_basis: _,
            partition_region_basis_digest: _,
            partition_region_basis_count: _,
            strategy_detail,
        } = self;
        match strategy_detail {
            ReuseStrategyBoundaryAuthority::None => {}
            ReuseStrategyBoundaryAuthority::CrossIdentity {
                persistent_correspondence_kind: _,
                persistent_correspondence_digest: _,
                persistent_correspondence_valid: _,
            } => {}
            ReuseStrategyBoundaryAuthority::PartialArtifactSplice {
                composition_region_digest: _,
                composition_region_count: _,
            } => {}
        }
        tolerance_regime
            .retained_heap_charge(work)?
            .checked_add(artifact_family.retained_heap_charge(work)?)
    }
}
