use super::super::{
    WorthQueryAdmittedDisclosedApplicationResult, WorthQueryApplicationQueryAccessReceipt,
    WorthQueryObservedResultSet, WorthQueryObservedSource,
};
use super::WorthQueryApplicationOneShotResult;

impl<Query, QueryResult> WorthQueryApplicationOneShotResult<Query, QueryResult> {
    /// Inspect the exact whole-orbit slot already disclosed by this result.
    pub fn orbit<Schema, Slot, Relation, Entity>(
        &self,
        reference: worth_query_declaration::facade::application_query::ApplicationQueryOrbitRef<
            Schema,
            Query,
            Slot,
            Relation,
            Entity,
        >,
    ) -> Option<super::super::WorthQueryApplicationQueryOrbit<'_, Schema, Relation, QueryResult>>
    {
        super::super::orbit_result::inspect(&self.rows, &self.result_set_observation, reference)
    }

    pub fn rows(&self) -> &[QueryResult] {
        &self.rows
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryAccessReceipt {
        &self.receipt
    }

    pub fn observed_sources(&self) -> &[WorthQueryObservedSource<Query>] {
        &self.observed_sources
    }

    pub fn result_set_observation(&self) -> &WorthQueryObservedResultSet<Query> {
        &self.result_set_observation
    }

    pub fn into_result_set_observation(self) -> WorthQueryObservedResultSet<Query> {
        self.result_set_observation
    }

    pub fn into_rows(self) -> Vec<QueryResult> {
        self.rows
    }
    pub fn into_admitted_disclosed(
        self,
    ) -> WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult> {
        WorthQueryAdmittedDisclosedApplicationResult::new_with_sources(
            self.rows,
            self.observed_sources,
            self.result_set_observation,
            self.request_affinity,
            self.receipt,
        )
    }
}
