//! Compare a real recording query's complete artifact, never only its partition digest.
use super::*;
impl WorthQueryHistoryOutputIntent {
    pub(in crate::domain_computation::primary_graph) fn matches_recording_source<Q>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::domain_computation::primary_graph::WorthQueryObservedSource<Q>,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<bool, Cause> {
        let _comparison = fund_comparison(
            execution,
            &self.backing.parameters,
            &source.parameters,
            scope_work(&self.backing.scope)?,
        )?;
        Ok(self.backing.query == source.query_identity
            && self.matches_output_source(source, observation)
            && self
                .backing
                .parameters
                .canonical_basis()
                .is_equivalent_to(source.parameters.canonical_basis()))
    }
}
