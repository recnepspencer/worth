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
    let (creation, encoding) = super::subject_binding::creation::<Schema, Binding>(&candidate)?;
    let subject_bytes = super::subject_binding::quote(creation, encoding)?;
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
    let metadata_bytes = [
        revision_text.as_str(),
        candidate.read_set.admission.operation(),
        Binding::IDENTITY,
        principal.issuer(),
        principal.subject(),
        state.slot.spec.purpose(),
        owner.metadata().verification_configuration(),
        owner.metadata().clock_source(),
        owner.metadata().clock_timeline(),
    ]
    .into_iter()
    .try_fold(4096usize, |total, text| {
        total.checked_add(text.len().checked_mul(6)?)
    })
    .ok_or(Denial::RepresentationCapacity)?;
    let provenance_bytes = metadata_bytes
        .checked_add(subject_bytes)
        .ok_or(Denial::RepresentationCapacity)?;
    let retained = provenance_bytes
        .checked_add(state.slot.field.owned_allocation_capacity_bytes())
        .ok_or(Denial::RepresentationCapacity)?;
    if retained > state.slot.representation_capacity {
        return Err(Denial::RepresentationCapacity);
    }
    let maximum_bytes = Binding::CANDIDATES
        .resources()
        .ok_or(Denial::RepresentationCapacity)?
        .maximum_retained_representation_bytes();
    let metadata_memory = execution
        .reserve(
            u64::try_from(
                provenance_bytes
                    .checked_mul(3)
                    .ok_or(Denial::RepresentationCapacity)?,
            )
            .map_err(|_| Denial::RepresentationCapacity)?,
        )
        .map_err(resource)?;
    let _canonical_memory = execution
        .reserve(
            u64::try_from(
                maximum_bytes
                    .checked_mul(6)
                    .ok_or(Denial::RepresentationCapacity)?,
            )
            .map_err(|_| Denial::RepresentationCapacity)?,
        )
        .map_err(resource)?;
    execution
        .decision_stage(|context| identity::admit_assembly(context, metadata_bytes))
        .map_err(Denial::Execution)??;
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
                identity::certified_subject(fields, subject, &certified_dependencies, maximum_bytes, &request, context)
            } else { identity::subject(
                fields,
                subject,
                &candidate.read_set.facts,
                maximum_bytes,
                &request,
                context,
            ) }
        })
        .map_err(Denial::Execution)??;
    let (effect_identity, candidate_work) = execution
        .decision_stage(|context| {
            context.checkpoint(1).map_err(stopped)?;
            identity::candidate(&candidate.effects, maximum_bytes, &request, context)
        })
        .map_err(Denial::Execution)??;
    let operation_meaning = *candidate
        .read_set
        .admission
        .installed_operation_identity_bytes();
    let observation = if encoding == worth_query_declaration::facade::application_schema::ApplicationSigningProvenanceEncoding::CertifiedSubjectV2 {
        let (digest, work) = execution.decision_stage(|context|
            super::super::dependency_identity::authentication_observation_identity(
                operation_meaning, &candidate.read_set.facts, maximum_bytes, &request, context,
            )).map_err(Denial::Execution)??;
        candidate.read_set.admission.retain_execution_canonical_work(work);
        Some(digest)
    } else { None };
    let (signing, signing_work) = execution
        .decision_stage(|context| {
            identity::admit_assembly(context, metadata_bytes)?;
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
                maximum_bytes,
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
        canonical_byte_capacity: maximum_bytes,
        provenance_bytes,
        encoding,
        certified_dependencies,
        _dependencies_memory: dependencies_memory,
        _metadata_memory: metadata_memory,
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

fn resource(
    cause: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial,
) -> Denial {
    Denial::Execution(
        crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause),
    )
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
