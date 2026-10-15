//! Deliberate native control. Ordinary fixed and source-only callers retain an operation instead.
use super::mutation::WorthQueryApplicationMutationRequest;
use super::WorthQueryApplicationRequest;
use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_installation::facade::ApplicationSchema;

/// Explicit request context for manual native staging, publication and output recovery.
pub struct WorthQueryAdvancedApplicationRequest<'application, 'principal, 'scope, Schema> {
    pub(in crate::application_entry) request:
        WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>,
}
impl<'a, 'p, 's, S: ApplicationSchema> WorthQueryAdvancedApplicationRequest<'a, 'p, 's, S> {
    pub fn mutate<I: ApplicationMutationIntent<S>>(
        &self,
        intent: I,
    ) -> WorthQueryApplicationMutationRequest<'a, 'p, 's, S, I> {
        WorthQueryApplicationMutationRequest::new(
            self.request.application,
            self.request.principal,
            self.request.scope,
            self.request.branch,
            intent,
        )
    }
}
