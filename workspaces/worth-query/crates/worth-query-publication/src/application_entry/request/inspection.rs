//! Inspect original query results and authored source operations with current access.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_execution::facade::application_inspection::{
    WorthQueryApplicationInspectionRequest, WorthQueryApplicationQueryObservation,
};

impl<'a, 'p, 'r, Schema: ApplicationSchema> WorthQueryApplicationRequest<'a, 'p, 'r, Schema> {
    /// Explain an authored operation from its original performed source receipt.
    ///
    /// Select `why_performed().reference_only()` to identify the operation, or
    /// `why_performed().materialize(...)` to read its recorded source effects.
    /// `Disclosure` names the operation's installed source disclosure binding.
    /// Each execution checks current inspection permission; no completed query
    /// or retained query projection is required.
    pub fn inspect_source<'source, Disclosure>(
        &self, source: &'source worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> worth_query_execution::facade::application_inspection::WorthQuerySourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>
    where Disclosure: worth_query_declaration::facade::application_operation::ApplicationSourceDisclosureBinding<Schema>{
        self.application.inspect_application_source::<Disclosure>(
            self.principal,
            self.scope,
            source,
        )
    }

    /// Explain a saved query result using the caller's current inspection access.
    ///
    /// Clone the observation from `result.receipt().inspect().observation()` before
    /// dropping the query result. Use `retain_projection_for_inspection()` on the
    /// query when you will compare or materialize its original values.
    ///
    /// Choose `why_observed()` for one original or `why_changed(&before)` for a
    /// comparison, then select `reference_only()` or `materialize(...)`. A producer
    /// explanation also needs the authored source receipt and a query read through
    /// `request.at(settlement.observation())`. Saving evidence does not preserve
    /// the caller's permission: each new inspection must admit current access.
    pub fn inspect<'o, Binding>(
        &self,
        original: &'o WorthQueryApplicationQueryObservation,
    ) -> WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        self.application.inspect_application_observation::<Binding>(
            self.principal,
            self.scope,
            original,
        )
    }
}
