//! What one mutation request binds into its commit beyond its own identities.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryBoundSourceExpectation,
};
use worth_query_execution::publication_boundary::workflow_advance::WorthQueryWorkflowAdvanceAdapter;
use worth_query_installation::facade::ApplicationSchema;

/// The source expectation the admission accepted and the workflow transition
/// the request advances, if any. Both extend the idempotency binding the
/// request's identities derive.
#[derive(Clone, Copy)]
pub(in crate::application_entry) struct WorthQueryCommitExtension {
    pub(super) source: Option<WorthQueryBoundSourceExpectation>,
    pub(super) workflow_transition: Option<[u8; 32]>,
}

impl WorthQueryCommitExtension {
    pub(in crate::application_entry) fn apply(
        self,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationIdempotencyBinding {
        let idempotency = match self.source {
            Some(source) => source.bind_idempotency(idempotency),
            None => idempotency,
        };
        match self.workflow_transition {
            Some(identity) => {
                WorthQueryWorkflowAdvanceAdapter::bind_operation_idempotency(idempotency, &identity)
            }
            None => idempotency,
        }
    }
}

/// The identities a request encoded once, with the extension its commit binds.
///
/// Every commit door of the request builds its idempotency binding from here,
/// so no door encodes the key or input again.
pub(in crate::application_entry) struct WorthQueryMutationCommitBinding<
    'identities,
    'request,
    Schema,
    Binding,
> where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    identities: &'identities ApplicationMutationIdentities<'request, Schema, Binding>,
    extension: WorthQueryCommitExtension,
}

impl<'identities, 'request, Schema, Binding>
    WorthQueryMutationCommitBinding<'identities, 'request, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::application_entry) fn new(
        identities: &'identities ApplicationMutationIdentities<'request, Schema, Binding>,
        extension: WorthQueryCommitExtension,
    ) -> Self {
        Self {
            identities,
            extension,
        }
    }

    pub(in crate::application_entry) fn identities(
        &self,
    ) -> &'identities ApplicationMutationIdentities<'request, Schema, Binding> {
        self.identities
    }

    pub(in crate::application_entry) fn extension(&self) -> WorthQueryCommitExtension {
        self.extension
    }

    /// The idempotency binding of the whole request.
    pub(in crate::application_entry) fn idempotency(
        &self,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.extension.apply(
            WorthQueryApplicationIdempotencyBinding::for_mutation_identities(self.identities),
        )
    }
}
