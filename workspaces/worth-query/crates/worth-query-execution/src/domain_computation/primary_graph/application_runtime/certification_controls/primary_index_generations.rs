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

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Evicts only the named field's rebuildable equality generations, not authoritative facts.
    #[doc(hidden)]
    pub fn release_equality_index_generations_for_test<E, A, F, V, W, Q, U>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        field: worth_query_installation::facade::ApplicationFieldRef<Schema, E, A, F, V, W, Q, U>,
    ) -> Result<usize, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        let execution = phase.execution_request_for(self.product_runtime())?;
        let graph = self
            .runtime
            .retain_primary_graph_integration_handle()
            .expect("installed primary graph");
        Ok(graph.with_query_runtime_mut(&execution, |runtime, layout| {
            let index = layout
                .equality_selector_index(field.entity(), field.aspect(), field.field())
                .expect("installed equality field");
            runtime
                .index_authority()
                .discard_generations(
                    worth_relational::facade::indexes::DerivedIndexDiscardRequest::all_bases(index),
                )
                .expect("installed selected equality index")
                .removed_generation_count()
        }))
    }
    /// Observes exact selected generation presence without preparing or reading the index.
    #[doc(hidden)]
    pub fn has_equality_index_generation_for_test<E, A, F, V, W, Q, U>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        branch: crate::basis::WorthQueryProductBranch,
        field: worth_query_installation::facade::ApplicationFieldRef<Schema, E, A, F, V, W, Q, U>,
    ) -> Result<bool, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        let execution = phase.execution_request_for(self.product_runtime())?;
        let selected = self
            .on_branch(branch)
            .select(phase)
            .expect("the observed branch selects");
        let graph = self
            .runtime
            .retain_primary_graph_integration_handle()
            .expect("installed primary graph");
        Ok(graph.with_query_runtime_mut(&execution, |runtime, layout| {
            let index = layout
                .equality_selector_index(field.entity(), field.aspect(), field.field())
                .expect("installed equality field");
            runtime
                .index_access()
                .published_generation_for_observation(
                    index,
                    &selected.product().relational_basis().observation(),
                )
                .is_some()
        }))
    }
}
