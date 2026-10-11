//! Eviction of rebuildable primary index projections for real read-stop proofs.
use super::*;
impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn release_primary_index_generations_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<usize, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        let execution = phase.execution_request_for(self.product_runtime())?;
        let graph = self
            .runtime
            .retain_primary_graph_integration_handle()
            .expect("installed primary graph");
        graph.with_runtime_mut(&execution, |runtime| {
            for index in graph.primary_index_ids.iter() {
                runtime
                    .index_authority()
                    .discard_generations(
                        worth_relational::facade::indexes::DerivedIndexDiscardRequest::all_bases(
                            *index,
                        ),
                    )
                    .expect("installed primary index can discard rebuildable generations");
            }
        });
        Ok(graph.primary_index_ids.len())
    }
}
