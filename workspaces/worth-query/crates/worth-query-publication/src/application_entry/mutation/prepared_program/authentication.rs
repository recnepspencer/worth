//! The request's original authenticated principal signs its already prepared candidate.
use super::{
    ApplicationMutationBinding, ApplicationProgramDefinition, ApplicationSchema,
    WorthQueryPreparedProgramMutation,
};
use worth_query_admission::facade::authentication_event::{
    WorthQueryAuthenticationEvent, WorthQueryAuthenticationEventDenial,
    WorthQueryAuthenticationEventIntent,
};
use worth_query_execution::publication_boundary::signed_creation::WorthQuerySignedCreationAdapter as Native;

impl<
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Program: ApplicationProgramDefinition<Schema>,
    > WorthQueryPreparedProgramMutation<'_, '_, Schema, Binding, Program>
{
    /// Derived from the actual candidate and its retained occurrence, never caller data.
    pub fn authentication_intent(&self) -> Option<&WorthQueryAuthenticationEventIntent> {
        Native::authentication_intent(&self.candidate.program)
    }

    /// Failure retains this candidate and leaves a mismatched event unconsumed.
    /// The signer cannot be replaced independently of the original request.
    pub fn sign(
        &mut self,
        event: &WorthQueryAuthenticationEvent<Schema>,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        Native::sign(&mut self.candidate.program, event, self.candidate.principal)
    }
}
