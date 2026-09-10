use super::ReuseBoundaryContext;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ReuseBoundaryContext {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            topology_regime: _,
            tolerance_regime,
            semantic_region,
            authority_policy: _,
            artifact_family,
            structural_dependency_basis: _,
            partition_region_basis,
            strategy_detail,
        } = self;
        tolerance_regime
            .retained_heap_charge(work)?
            .checked_add(semantic_region.retained_heap_charge(work)?)?
            .checked_add(artifact_family.retained_heap_charge(work)?)?
            .checked_add(partition_region_basis.retained_heap_charge(work)?)?
            .checked_add(strategy_detail.retained_heap_charge(work)?)
    }
}
