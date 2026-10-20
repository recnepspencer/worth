//! Mandatory fresh-commit gate shared by ordinary, advanced and platform effect paths.
use super::super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitOutcome as Outcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationRealizedEffect as Effect,
};
use super::{identity, SignedCreationState};
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial as AuthenticationDenial;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, ApplicationSigningEntityCoordinates,
    ApplicationSigningGraphSubjectProvenance, ApplicationSigningGraphSubjectProvenanceBinding,
    ApplicationSigningGraphSubjectProvenanceParts, ApplicationSigningProvenance,
    ApplicationSigningProvenanceBinding, ApplicationSigningProvenanceParts,
    ApplicationSigningSubjectProvenance, ApplicationSigningSubjectProvenanceBinding,
    ApplicationSigningSubjectProvenanceParts,
};

pub(in crate::domain_computation::primary_graph::application_attempt) fn publish<Schema>(
    state: Option<SignedCreationState<Schema>>,
    effects: &mut [Effect],
    execution: &QueryRequestExecution<'_>,
    lease: &super::super::snapshot_lease::WorthQueryApplicationSnapshotLease,
    idempotency: &WorthQueryApplicationIdempotencyBinding,
) -> Result<(), Outcome> {
    let request = execution.execution_request().map_err(|cause| {
        crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause)
            .into_commit_outcome()
    })?;
    execution
        .decision_stage(|context| {
            publish_prepared(state, effects, &request, lease, idempotency, context)
        })
        .map_err(|cause| cause.into_commit_outcome())?
        .map_err(PublicationRefusal::into_outcome)
}

fn publish_prepared<Schema>(
    state: Option<SignedCreationState<Schema>>,
    effects: &mut [Effect],
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    lease: &super::super::snapshot_lease::WorthQueryApplicationSnapshotLease,
    idempotency: &WorthQueryApplicationIdempotencyBinding,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), PublicationRefusal> {
    let layout = &lease.layout;
    let signed_creates = validate_effects(effects, execution, lease)?;
    let Some(state) = state else {
        return if signed_creates == 0 {
            Ok(())
        } else {
            Err(refused(AuthenticationDenial::MissingSigningProof).into())
        };
    };
    let sealed = state
        .sealed
        .ok_or_else(|| refused(AuthenticationDenial::MissingSigningProof))?;
    let consumed = state
        .authentication
        .ok_or_else(|| refused(AuthenticationDenial::MissingSigningProof))?;
    if signed_creates != 1
        || idempotency.key_identity() != &sealed.logical_request
        || idempotency.intent_identity() != &sealed.input
        || !idempotency.is_for_mutation_binding(&sealed.metadata.binding)
    {
        return Err(refused(AuthenticationDenial::IntentMismatch).into());
    }
    let (actual, _) =
        identity::candidate(effects, execution, context).map_err(PublicationRefusal::canonical)?;
    if actual != sealed.effect_identity
        || consumed.principal() != sealed.principal.as_ref()
        || consumed.intent() != &sealed.intent
    {
        return Err(refused(AuthenticationDenial::IntentMismatch).into());
    }
    let Some(Effect::CreateEntity { kind, fields, .. }) = effects.get_mut(state.slot.effect) else {
        return Err(refused(AuthenticationDenial::MissingSigningProof).into());
    };
    if !layout.is_signed_entity_kind(*kind) || fields.contains_key(&state.slot.field) {
        return Err(refused(AuthenticationDenial::MissingSigningProof).into());
    }
    let consumed = sealed
        .owner
        .readmit_for_publication(consumed)
        .map_err(refused)?;
    if consumed.verification_configuration() != sealed.metadata.verification_configuration
        || consumed.source_identity().as_str() != sealed.metadata.clock_source
        || consumed.timeline_identity().as_str() != sealed.metadata.clock_timeline
    {
        return Err(refused(AuthenticationDenial::WrongOwner).into());
    }
    let parts = ApplicationSigningProvenanceParts {
        metadata: sealed.metadata,
        operation_meaning: sealed.operation_meaning,
        logical_request: sealed.logical_request,
        input: sealed.input,
        subject: coordinates(sealed.subject),
        signer: coordinates(sealed.signer),
        signing_intent: *sealed.intent.signing_intent().bytes(),
        subject_coverage: *sealed.intent.subject_coverage().bytes(),
        event_reference: *consumed.event_reference(),
        issued_at_nanoseconds: consumed.issued_at_nanoseconds(),
        observed_at_nanoseconds: consumed.observed_at_nanoseconds(),
    };
    let encoded = match sealed.subject_binding {
        super::subject_binding::CapturedSubject::Authentication => {
            ApplicationSigningProvenance::from_parts(parts)
                .map_err(|_| refused(AuthenticationDenial::MissingSigningProof))
                .and_then(|value| {
                    ApplicationSigningProvenanceBinding::encode(&value)
                        .map_err(|_| refused(AuthenticationDenial::MissingSigningProof))
                })
        }
        super::subject_binding::CapturedSubject::Scope(dependencies) => {
            ApplicationSigningSubjectProvenance::from_parts(
                ApplicationSigningSubjectProvenanceParts {
                    authentication: parts,
                    dependencies,
                },
            )
            .map_err(|_| refused(AuthenticationDenial::MissingSigningProof))
            .and_then(|value| {
                ApplicationSigningSubjectProvenanceBinding::encode(&value)
                    .map_err(|_| refused(AuthenticationDenial::MissingSigningProof))
            })
        }
        super::subject_binding::CapturedSubject::Graph {
            dependencies,
            selections,
        } => ApplicationSigningGraphSubjectProvenance::from_parts(
            ApplicationSigningGraphSubjectProvenanceParts {
                authentication: parts,
                dependencies,
                selections,
            },
        )
        .map_err(|_| refused(AuthenticationDenial::MissingSigningProof))
        .and_then(|value| {
            ApplicationSigningGraphSubjectProvenanceBinding::encode(&value)
                .map_err(|_| refused(AuthenticationDenial::MissingSigningProof))
        }),
    }
    .map_err(|_| refused(AuthenticationDenial::MissingSigningProof))?;
    fields.insert(state.slot.field, encoded);
    Ok(())
}

fn validate_effects(
    effects: &[Effect],
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    lease: &super::super::snapshot_lease::WorthQueryApplicationSnapshotLease,
) -> Result<usize, Denial> {
    let layout = &lease.layout;
    if !layout.has_signed_entity_kinds() {
        return Ok(0);
    }
    let mut creates = 0;
    for effect in effects {
        match effect {
            Effect::CreateEntity { kind, .. } if layout.is_signed_entity_kind(*kind) => {
                creates += 1
            }
            Effect::UpdateEntity { entity_id, .. }
            | Effect::PatchOptionalEntityFields { entity_id, .. }
            | Effect::DeleteEntity { entity_id } => {
                let kind = lease.handle().with_runtime(execution, |runtime| {
                    let view = runtime
                        .read_truth()
                        .project_snapshot(lease.snapshot())
                        .ok_or_else(Denial::signed_entity_immutable)?;
                    view.entity_record_with_projection_scope(
                        execution,
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| Some(record.kind_id()),
                    )
                    .map_err(|cause| {
                        Denial::provider_execution_denied(
                            super::super::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
                            crate::facade::primary_graph::relational_execution_kind(cause, None),
                            "signed entity kind observation stopped",
                        )
                    })?
                    .ok_or_else(Denial::signed_entity_immutable)
                })?;
                if layout.is_signed_entity_kind(kind) {
                    return Err(Denial::signed_entity_immutable());
                }
                if layout.signing_lifecycle.is_record(kind) {
                    return Err(Denial::signing_lifecycle(super::super::WorthQueryApplicationCommitDenialKind::SigningLifecycleForgery));
                }
            }
            _ => {}
        }
    }
    Ok(creates)
}
fn refused(kind: AuthenticationDenial) -> Denial {
    Denial::signed_creation_authentication(kind)
}
fn coordinates(
    entity: worth_relational::facade::identity::EntityId,
) -> ApplicationSigningEntityCoordinates {
    ApplicationSigningEntityCoordinates {
        partition: entity.partition_value_u64(),
        slot: entity.local_slot_value(),
        generation: entity.generation_value(),
    }
}

enum PublicationRefusal {
    Denied(Denial),
    Execution(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
}
impl From<Denial> for PublicationRefusal {
    fn from(value: Denial) -> Self {
        Self::Denied(value)
    }
}
impl PublicationRefusal {
    fn canonical(cause: super::sealing::WorthQuerySignedCreationPreparationDenial) -> Self {
        match cause {
            super::sealing::WorthQuerySignedCreationPreparationDenial::Execution(cause) => {
                Self::Execution(cause)
            }
            _ => Self::Denied(refused(AuthenticationDenial::IntentMismatch)),
        }
    }
    fn into_outcome(self) -> Outcome {
        match self {
            Self::Denied(cause) => Outcome::Denied(cause),
            Self::Execution(cause) => cause.into_commit_outcome(),
        }
    }
}
