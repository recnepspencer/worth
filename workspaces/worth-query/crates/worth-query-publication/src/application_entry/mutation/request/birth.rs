//! Exact birth requirement binding, separate from an existing occurrence.
use super::{
    ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationIdentities,
    ApplicationMutationIdentityDenial, ApplicationMutationIntent, ApplicationSchema,
    WorthQueryApplicationMutationRequestWithIdempotency,
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
    pub(in crate::application_entry) fn bind_workflow_birth(
        mut self,
        identity: [u8; 32],
        input: ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        authority: std::sync::Arc<
            worth_query_execution::publication_boundary::workflow_birth::WorkflowBirthAuthoritySlot,
        >,
    ) -> Self {
        self.workflow_birth_identity = Some(identity);
        self.workflow_input = Some(input);
        self.workflow_birth_authority = Some(authority);
        self
    }

    pub(in crate::application_entry) const fn workflow_birth_identity(&self) -> Option<[u8; 32]> {
        self.workflow_birth_identity
    }

    // The owning request view already borrows its frozen input. Recovery binds
    // the original identity without consuming the birth's one-shot authority.
    pub(in crate::application_entry) fn bind_retained_workflow_birth(
        &mut self,
        identity: [u8; 32],
        authority: Option<std::sync::Arc<worth_query_execution::publication_boundary::workflow_birth::WorkflowBirthAuthoritySlot>>,
    ) {
        self.workflow_birth_identity = Some(identity);
        self.workflow_birth_authority = authority;
    }

    pub(in crate::application_entry) fn identities_with_input<'identity>(
        &'identity self,
        input: &'identity ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
    ) -> Result<
        ApplicationMutationIdentities<'identity, Schema, Intent::Binding>,
        ApplicationMutationIdentityDenial,
    > {
        ApplicationMutationIdentities::with_encoded_input(self.key, input)
    }
}
