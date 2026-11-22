use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::RelationalSourceOwnerInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema + 'static> super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Read-only physical candidate-input observation for product integration
    /// tests; it does not expose a Relational runtime or graph read authority.
    /// Must not be called inside `with_runtime`: it takes the source owner lock.
    #[doc(hidden)]
    pub fn custom_invariant_candidate_input_counters(
        &self,
    ) -> worth_relational::facade::runtime::RelationalCandidateInputCounters {
        self.runtime
            .primary_provider
            .graph_ref()
            .source_owner_ref()
            .custom_invariant_candidate_input_counters()
    }
}
