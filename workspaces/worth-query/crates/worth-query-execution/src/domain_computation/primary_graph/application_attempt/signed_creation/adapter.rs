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
    /// A joint native publication may rely on this exact candidate's proof.
    /// It neither consumes another event nor turns descriptive provenance into authority.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn validate_shared_authentication<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        candidate: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        intent: &WorthQueryAuthenticationEventIntent,
        principal: &worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        let state = candidate
            .read_set
            .signed_creation
            .as_ref()
            .ok_or(WorthQueryAuthenticationEventDenial::MissingSigningProof)?;
        let sealed = state
            .sealed
            .as_ref()
            .ok_or(WorthQueryAuthenticationEventDenial::MissingSigningProof)?;
        let proof = state
            .authentication
            .as_ref()
            .ok_or(WorthQueryAuthenticationEventDenial::MissingSigningProof)?;
        if proof.principal() != principal || sealed.principal.as_ref() != principal {
            return Err(WorthQueryAuthenticationEventDenial::PrincipalMismatch);
        }
        if proof.intent() != intent || &sealed.intent != intent {
            return Err(WorthQueryAuthenticationEventDenial::IntentMismatch);
        }
        Ok(())
    }
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
