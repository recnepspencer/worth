//! Real disposal and admitted reconstruction of the canonical correction projections.
use super::*;
use worth_relational::facade::indexes::{
    DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceOutcome,
};

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Discards only derived correction generations, preserving every canonical fact.
    #[doc(hidden)]
    pub fn release_correction_history_indexes_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<usize, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        let execution = phase.execution_request_for(self.product_runtime())?;
        let graph = self
            .runtime
            .retain_primary_graph_integration_handle()
            .expect("installed primary graph");
        let lineage = &graph.layout.provider_aftermath_causality().lineage;
        let indexes = [
            lineage.reversal_root_index,
            lineage.reversed_source_index,
            lineage.consumed_reversal_index,
        ];
        Ok(graph.with_runtime_mut(&execution, |runtime| indexes.into_iter().map(|index| runtime.index_authority().discard_generations(worth_relational::facade::indexes::DerivedIndexDiscardRequest::all_bases(index)).expect("installed correction index").removed_generation_count()).sum()))
    }

    /// Uses the existing native reconstruction owner against an exactly selected basis.
    #[doc(hidden)]
    pub fn rebuild_correction_history_indexes_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let execution = phase
            .execution_request_for(self.product_runtime())
            .expect("control uses its receiver's phase");
        let selected = self
            .on_branch(branch)
            .select(phase)
            .expect("control uses a live branch");
        let basis = selected.product().relational_basis();
        let graph = self
            .runtime
            .retain_primary_graph_integration_handle()
            .expect("installed primary graph");
        let lineage = &graph.layout.provider_aftermath_causality().lineage;
        let index_ids = vec![
            lineage.reversal_root_index,
            lineage.reversed_source_index,
            lineage.consumed_reversal_index,
        ];
        graph.with_runtime_mut(&execution, |runtime| runtime.index_authority().refresh_for_basis(
            worth_relational::facade::indexes::DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().expect("published correction basis"),
                branch_id: basis.identity().branch_id().clone(), index_ids,
            }, basis, None, crate::domain_computation::primary_graph::index_maintenance_budget::cold_index_reconstruction_budget(), &execution,
        ))
    }
}
