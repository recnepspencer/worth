use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema + 'static> super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Read-only physical candidate-input observation for product integration
    /// tests; it does not expose a Relational runtime or graph read authority.
    /// Must not be called inside `with_runtime`: it takes the source owner lock.
    #[doc(hidden)]
    pub fn custom_invariant_candidate_input_counters(
        &self,
    ) -> worth_relational::facade::runtime::RelationalCandidateInputCounters {
        self.primary_provider
            .graph
            .source_owner
            .custom_invariant_candidate_input_counters()
    }
}
