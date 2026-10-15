//! Publication-only access to the existing candidate's native signing progression.
use super::{sealing, WorthQuerySignedCreationPreparationDenial};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationEffectProgram,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_admission::facade::{
    authenticated_principal::WorthQueryAuthenticatedExternalPrincipal,
    authentication_event::{
        WorthQueryAuthenticationEvent, WorthQueryAuthenticationEventDenial,
        WorthQueryAuthenticationEventIntent,
    },
};
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIdentities},
    application_schema::ApplicationSchema,
};

pub struct WorthQuerySignedCreationAdapter;
impl WorthQuerySignedCreationAdapter {
    pub fn prepare<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        candidate: sealing::BodyCandidate<Schema, Binding>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    ) -> Result<sealing::BodyCandidate<Schema, Binding>, WorthQuerySignedCreationPreparationDenial>
    {
        sealing::seal(phase, application, candidate, identities)
    }
    pub fn authentication_intent<Schema, Operation, Input, Scope>(
        candidate: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    ) -> Option<&WorthQueryAuthenticationEventIntent> {
        sealing::intent(candidate)
    }
    pub fn sign<Schema, Operation, Input, Scope>(
        candidate: &mut WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        event: &WorthQueryAuthenticationEvent<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        sealing::sign(candidate, event, principal)
    }
}
