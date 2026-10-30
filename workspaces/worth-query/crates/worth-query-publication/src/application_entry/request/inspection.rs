//! Host forwards the original observation and fresh request to Execution.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_execution::facade::application_inspection::{
    WorthQueryApplicationInspectionRequest, WorthQueryApplicationQueryObservation,
};

impl<'a, 'p, 'r, Schema: ApplicationSchema> WorthQueryApplicationRequest<'a, 'p, 'r, Schema> {
    /// A performed source uses its own installed disclosure and fresh authority;
    /// no completed query or retained projection is required.
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
