//! Closed D1634 forwarding entry: real Native symbol residency placement only.
use super::*;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn place_symbol_pages_cold_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        worth_relational::facade::runtime::RelationalColdSymbolPlacementForTest,
        crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
    > {
        let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
        Ok(self
            .runtime
            .primary_provider
            .graph
            .with_runtime_mut(&execution, |runtime| {
                runtime.place_symbol_pages_cold_for_test(&execution)
            }))
    }
}
