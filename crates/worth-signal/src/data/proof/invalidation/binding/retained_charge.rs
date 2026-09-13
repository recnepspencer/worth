use super::{
    DependencyCauseBindingAxes, DependencyCauseKey, PendingDependencyRevalidation,
    ResolvedDependencyCause,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for PendingDependencyRevalidation {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            dependency_revision: _,
            unresolved_producers,
            requires_structural_recompute: _,
        } = self;
        unresolved_producers.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ResolvedDependencyCause {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            key,
            binding_axes,
            changed_scopes,
        } = self;
        let DependencyCauseKey {
            graph_instance: _,
            consumer: _,
            dependency_revision: _,
            producer: _,
            aspect: _,
            edge_scope,
        } = key;
        let DependencyCauseBindingAxes {
            graph_instance: _,
            consumer: _,
            dependency_revision: _,
            producer: _,
            aspect: _,
            edge_scope: binding_scope,
            cached_version: _,
            output_commit_ordinal: _,
            committed_version: _,
        } = binding_axes;
        edge_scope
            .retained_heap_charge(work)?
            .checked_add(binding_scope.retained_heap_charge(work)?)?
            .checked_add(changed_scopes.retained_heap_charge(work)?)
    }
}
