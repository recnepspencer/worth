use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{
    ArtifactEquivalenceContract, ArtifactSemanticBoundary, NodeReuseContract, ReuseStrategy,
};

impl RetainedStorageMeasurement for ArtifactEquivalenceContract {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            required_boundaries,
            supported_strategies,
            allows_snapshot_restore_reuse: _,
            allows_authority_reconciliation_reuse: _,
        } = self;
        // Both element types are closed scalar enums: only the allocated slots own bytes.
        Charge::capacity::<ArtifactSemanticBoundary>(required_boundaries.capacity())?.checked_add(
            Charge::capacity::<ReuseStrategy>(supported_strategies.capacity())?,
        )
    }
}

impl RetainedStorageMeasurement for NodeReuseContract {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            equivalence,
            retain_certification: _,
        } = self;
        Ok(Charge::ZERO.checked_add(equivalence.retained_heap_charge(work)?)?)
    }
}
