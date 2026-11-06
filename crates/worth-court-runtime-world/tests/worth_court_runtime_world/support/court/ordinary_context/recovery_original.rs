//! Release requests carry the actual Native artifact retained by World.
use super::CompositeSupplyChainCourt;
impl CompositeSupplyChainCourt {
    pub(crate) fn observe_recovery_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
    ) -> Option<worth_relational::facade::durability::ObservedRelationalMemoryCommit> {
        let effects = self.world.recovery_port().inspect_effects(handle).unwrap();
        effects
            .component_results()
            .relational_settlement()
            .map(|receipt| {
                self.records
                    .runtime
                    .durability_authority()
                    .observe_memory_commit(execution, receipt)
                    .unwrap()
            })
    }
}
