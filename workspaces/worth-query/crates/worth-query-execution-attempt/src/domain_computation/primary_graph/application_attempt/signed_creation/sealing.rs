//! Seals the actual prepared native body once, at its retained occurrence snapshot.
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationEffectProgram,
};
use super::{identity, SealedSigningContext};
use crate::domain_computation::primary_graph::{
};
use std::sync::Arc;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::authentication_event::{
    WorthQueryAuthenticationEventDenial, WorthQueryAuthenticationEventIntent,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
    },
    application_schema::{ApplicationSchema, ApplicationSigningProvenanceMetadata},
};

pub type BodyCandidate<Schema, Binding> = WorthQueryApplicationEffectProgram<Schema, <Binding as ApplicationMutationBinding<Schema>>::Operation, <Binding as ApplicationMutationBinding<Schema>>::Input, <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope>;

#[derive(Debug)]
/// Why the native owner could not seal a signed creation at its retained occurrence.
pub enum WorthQuerySignedCreationPreparationDenial {
    Execution(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    Commit(WorthQueryApplicationCommitDenial),
    LifecycleObservation(Box<super::super::WorthQueryApplicationCommitOutcome>),
    Attempt(WorthQueryApplicationAttemptDenial),
    Authentication(WorthQueryAuthenticationEventDenial),
    OwnerUnavailable,
    CandidateMismatch,
    RepresentationCapacity,
    Canonicalization,
    CanonicalBasis(worth_foundational::facade::CanonicalBasisConstructionDenial),
    CanonicalDigest(worth_foundational::facade::CanonicalDigestDerivationDenial),
    CanonicalAccountingOverflow,
    CanonicalAllocationUnavailable,
}
impl std::fmt::Display for WorthQuerySignedCreationPreparationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "signed creation preparation denied: {self:?}")
    }
}
impl std::error::Error for WorthQuerySignedCreationPreparationDenial {}
pub type Denial = WorthQuerySignedCreationPreparationDenial;


pub fn intent<Schema, Operation, Input, Scope>(
    candidate: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
) -> Option<&WorthQueryAuthenticationEventIntent> {
    candidate
        .read_set
        .signed_creation
        .as_ref()?
        .sealed
        .as_ref()
        .map(|context| &context.intent)
}

pub fn sign<Schema, Operation, Input, Scope>(
    candidate: &mut WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    event: &worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEvent<
        Schema,
    >,
    principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
) -> Result<(), WorthQueryAuthenticationEventDenial> {
    let state = candidate
        .read_set
        .signed_creation
        .as_mut()
        .ok_or(WorthQueryAuthenticationEventDenial::MissingSigningProof)?;
    let sealed = state
        .sealed
        .as_ref()
        .ok_or(WorthQueryAuthenticationEventDenial::MissingSigningProof)?;
    if state.authentication.is_some() {
        return Err(WorthQueryAuthenticationEventDenial::EventUnavailable);
    }
    if principal.identity() != sealed.principal.as_ref() {
        return Err(WorthQueryAuthenticationEventDenial::PrincipalMismatch);
    }
    state.authentication = Some(sealed.owner.consume_for_signing(
        event,
        principal,
        &sealed.intent,
        candidate.read_set.admission.publication_request(),
    )?);
    Ok(())
}
