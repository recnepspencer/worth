//! Seals the actual prepared native body once, at its retained occurrence snapshot.
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationEffectProgram,
};
use super::{identity, SealedSigningContext};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
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
    CanonicalSortUnsupported,
    CanonicalAllocationUnavailable,
}
impl std::fmt::Display for WorthQuerySignedCreationPreparationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "signed creation preparation denied: {self:?}")
    }
}
impl std::error::Error for WorthQuerySignedCreationPreparationDenial {}
type Denial = WorthQuerySignedCreationPreparationDenial;

pub(in crate::domain_computation::primary_graph) fn seal<
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
>(
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    candidate: BodyCandidate<Schema, Binding>,
    identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
) -> Result<BodyCandidate<Schema, Binding>, Denial> {
    let Some(state) = candidate.read_set.signed_creation.as_ref() else {
        return Ok(candidate);
    };
    if state.sealed.is_some() {
        return Err(Denial::CandidateMismatch);
    }
    let execution = phase
        .execution_for(&application.product_runtime)
        .map_err(|cause| Denial::Execution(cause.into()))?;
    let candidate = super::super::signing_lifecycle::capture_signing(execution, candidate)?;
    let (mut candidate, dependencies_memory) = super::dependencies::capture(execution, candidate)?;
    let state = candidate
        .read_set
        .signed_creation
        .as_ref()
        .expect("retained native signing slot");
    let (_, encoding) = super::subject_binding::creation::<Schema, Binding>(&candidate)?;
    let request = phase
        .execution_request_for(&application.product_runtime)
        .map_err(|_| Denial::CandidateMismatch)?;
    candidate
        .read_set
        .admission
        .validate_current_authority()
        .map_err(|cause| {
            Denial::Attempt(WorthQueryApplicationAttemptDenial::request_authority_lost(
                cause,
            ))
        })?;
    let support = application
        .installed_program_support()
        .ok_or(Denial::OwnerUnavailable)?;
    let occurrence =
        super::super::provider_execution::resolve_occurrence_program(&request, support, &candidate)
            .map_err(Denial::Commit)?;
    let revision = *occurrence.entry().revision();
    let owner = application
        .signing_owners
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&revision)
        .cloned()
        .ok_or(Denial::OwnerUnavailable)?;
    let principal = candidate
        .read_set
        .admission
        .signed_creation_principal()
        .map(Arc::clone)
        .ok_or(Denial::Authentication(
            WorthQueryAuthenticationEventDenial::PrincipalMismatch,
        ))?;
    let revision_text = revision.to_string();
    let metadata = ApplicationSigningProvenanceMetadata {
        program_revision: revision_text,
        operation: candidate.read_set.admission.operation().to_owned(),
        binding: Binding::IDENTITY.to_owned(),
        signer_issuer: principal.issuer().to_owned(),
        signer_subject: principal.subject().to_owned(),
        purpose: state.slot.spec.purpose().to_owned(),
        verification_configuration: owner.metadata().verification_configuration().to_owned(),
        clock_source: owner.metadata().clock_source().to_owned(),
        clock_timeline: owner.metadata().clock_timeline().to_owned(),
    };
    let super::super::WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. } = candidate
        .effects
        .get(state.slot.effect)
        .ok_or(Denial::CandidateMismatch)?
    else {
        return Err(Denial::CandidateMismatch);
    };
    if fields.contains_key(&state.slot.field) {
        return Err(Denial::CandidateMismatch);
    }
    let subject = candidate.read_set.admission.scope_entity_id();
    let signer = candidate.read_set.admission.graph_work().principal();
    let certified_dependencies = execution
        .decision_stage(|context| {
            super::subject_binding::capture::<Schema, Binding>(&candidate, context)
        })
        .map_err(Denial::Execution)??;
    let (coverage, coverage_work) = execution
        .decision_stage(|context| {
            context.checkpoint(1).map_err(stopped)?;
            if encoding == worth_query_declaration::facade::application_schema::ApplicationSigningProvenanceEncoding::CertifiedSubjectV2 {
                identity::certified_subject(fields, subject, &certified_dependencies, &request, context)
            } else { identity::subject(
                fields,
                subject,
                &candidate.read_set.facts,
                &request,
                context,
            ) }
        })
        .map_err(Denial::Execution)??;
    let (effect_identity, candidate_work) = execution
        .decision_stage(|context| {
            context.checkpoint(1).map_err(stopped)?;
            identity::candidate(&candidate.effects, &request, context)
        })
        .map_err(Denial::Execution)??;
    let operation_meaning = *candidate
        .read_set
        .admission
        .installed_operation_identity_bytes();
    let observation = if encoding == worth_query_declaration::facade::application_schema::ApplicationSigningProvenanceEncoding::CertifiedSubjectV2 {
        let (digest, work) = execution.decision_stage(|context|
            super::super::dependency_identity::authentication_observation_identity(
                operation_meaning, &candidate.read_set.facts, &request, context,
            )).map_err(Denial::Execution)??;
        candidate.read_set.admission.retain_execution_canonical_work(work);
        Some(digest)
    } else { None };
    let (signing, signing_work) = execution
        .decision_stage(|context| {
            context.checkpoint(0).map_err(stopped)?;
            let mut signing = identity::IdentityEntries::new();
            signing.text("program", revision.to_string());
            signing.text("binding", Binding::IDENTITY);
            signing.text("purpose", state.slot.spec.purpose());
            signing.digest("operation", operation_meaning);
            signing.digest("logical-request", *identities.key_identity());
            signing.digest("input", *identities.input_identity());
            signing.digest("subject", coverage);
            signing.digest("candidate", effect_identity);
            if let Some(observation) = observation {
                signing.digest("observation", observation);
            }
            signing.entity("signer", signer);
            signing.text("external-issuer", principal.issuer());
            signing.text("external-subject", principal.subject());
            signing.derive(
                if observation.is_some() {
                    "worth-query-signed-certified-creation-intent-v2"
                } else {
                    "worth-query-signed-creation-intent-v1"
                },
                &request,
                context,
            )
        })
        .map_err(Denial::Execution)??;
    let intent = WorthQueryAuthenticationEventIntent::new(
        state.slot.spec.purpose(),
        CanonicalDigestId::new(signing),
        CanonicalDigestId::new(coverage),
    )
    .map_err(|_| Denial::CandidateMismatch)?;
    for work in [coverage_work, candidate_work, signing_work] {
        candidate
            .read_set
            .admission
            .retain_execution_canonical_work(work);
    }
    candidate
        .read_set
        .signed_creation
        .as_mut()
        .expect("retained native slot")
        .sealed = Some(SealedSigningContext {
        encoding,
        certified_dependencies,
        _dependencies_memory: dependencies_memory,
        owner,
        principal,
        intent,
        metadata,
        logical_request: *identities.key_identity(),
        input: *identities.input_identity(),
        operation_meaning,
        effect_identity,
        subject,
        signer,
    });
    Ok(candidate)
}

fn stopped(cause: worth_execution::MapKernelStop) -> Denial {
    Denial::Execution(
        crate::domain_computation::primary_graph::application_contribution::advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(cause),
        ),
    )
}

pub(in crate::domain_computation::primary_graph) fn intent<Schema, Operation, Input, Scope>(
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

pub(in crate::domain_computation::primary_graph) fn sign<Schema, Operation, Input, Scope>(
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
