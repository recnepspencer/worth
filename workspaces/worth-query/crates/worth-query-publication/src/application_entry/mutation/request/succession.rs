//! Retain an exact succession handoff on the ordinary mutation request.
use super::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationIntent,
    ApplicationSchema, WorthQueryApplicationMutationRequestWithIdempotency,
};
impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) fn bind_workflow_succession(
        mut self,
        identity: [u8; 32],
        input: ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        authority: std::sync::Arc<worth_query_execution::publication_boundary::workflow_succession::WorkflowSuccessionAuthoritySlot>,
    ) -> Self {
        self.workflow_succession_identity = Some(identity);
        self.workflow_input = Some(input);
        self.workflow_succession_authority = Some(authority);
        self
    }
    pub(in crate::application_entry) const fn workflow_succession_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.workflow_succession_identity
    }
}
