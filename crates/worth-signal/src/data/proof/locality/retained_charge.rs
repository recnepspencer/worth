use super::PartitionScopeSet;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for PartitionScopeSet {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for super::DedupedNodeBatch {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { nodes } = self;
        nodes.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for super::SortedSourceBatch {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { sources } = self;
        sources.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for super::TouchedScopeSummary {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            seed_scopes,
            inclusion_scopes,
            direct_dirty_scopes,
            maybe_stale_scopes,
            touched_nodes,
            touched_sources,
        } = self;
        seed_scopes
            .retained_heap_charge(work)?
            .checked_add(inclusion_scopes.retained_heap_charge(work)?)?
            .checked_add(direct_dirty_scopes.retained_heap_charge(work)?)?
            .checked_add(maybe_stale_scopes.retained_heap_charge(work)?)?
            .checked_add(touched_nodes.retained_heap_charge(work)?)?
            .checked_add(touched_sources.retained_heap_charge(work)?)
    }
}
