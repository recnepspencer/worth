//! Host forwards the original observation and fresh request to Execution.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_execution::facade::causal_inspection::{
    WorthQueryApplicationInspectionRequest, WorthQueryApplicationQueryObservation,
};

impl<'a, 'p, 'r, Schema: ApplicationSchema> WorthQueryApplicationRequest<'a, 'p, 'r, Schema> {
    /// Starts a fresh inspection of the original observation's current branch.
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
