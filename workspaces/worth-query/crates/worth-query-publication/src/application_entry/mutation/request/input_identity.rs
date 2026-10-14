//! Request identities reuse immutable encoded operation input custody.
use super::*;

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
    /// The request's key and input with the identities encoded from exactly them.
    ///
    /// Each entry point calls this once and passes the result through preparation,
    /// execution, and the idempotency binding; the identities borrow the key and
    /// the input this request owns, so they cannot be stored beside them. A
    /// request bound to a workflow requirement already encoded its input when it
    /// bound, so only the key is encoded here.
    pub(in crate::application_entry) fn identities(
        &self,
    ) -> Result<
        ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        if let Some(input) = self.operation_input {
            let input = input.as_ref().map_err(|denial| WorthQueryApplicationRequestMutationDenial::Identity(worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial::Input(denial.clone())))?;
            return ApplicationMutationIdentities::with_encoded_input(self.key, input)
                .map_err(WorthQueryApplicationRequestMutationDenial::Identity);
        }
        match &self.workflow_input {
            Some(input) => ApplicationMutationIdentities::with_encoded_input(self.key, input),
            None => ApplicationMutationIdentities::encode(self.key, self.request.intent.input()),
        }
        .map_err(WorthQueryApplicationRequestMutationDenial::Identity)
    }

    /// Encodes the request's input once, for binding it to a workflow
    /// requirement; the entry point later reuses it through `identities`.
    pub(in crate::application_entry) fn encode_input(
        &self,
    ) -> Result<
        ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        ApplicationMutationIdentityDenial,
    >
    where
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone,
    {
        ApplicationEncodedInput::encode(self.request.intent.input().clone())
            .map_err(ApplicationMutationIdentityDenial::Input)
    }
}
