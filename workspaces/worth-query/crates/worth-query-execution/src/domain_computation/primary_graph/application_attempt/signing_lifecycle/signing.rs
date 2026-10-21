//! Signing admits and retains exact current membership before authentication, including absence.
use super::super::signed_creation::WorthQuerySignedCreationPreparationDenial as SigningDenial;
use super::super::WorthQueryApplicationEffectProgram;
use super::{observation, Denial, Kind, QueryRequestExecution};

pub(in crate::domain_computation::primary_graph::application_attempt) fn capture_signing<
    Schema,
    Operation,
    Input,
    Scope,
>(
    execution: &QueryRequestExecution<'_>,
    mut candidate: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
) -> Result<WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>, SigningDenial> {
    let Some(spec) = candidate
        .read_set
        .admission
        .allowed_graph_contract()
        .body()
        .and_then(|body| body.creation())
        .and_then(|creation| creation.signing_lifecycle())
    else {
        return Ok(candidate);
    };
    let lease = &candidate.read_set.lease;
    let request = execution.execution_request().map_err(|cause| {
        SigningDenial::Execution(
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause),
        )
    })?;
    let limit = spec.maximum_evidence;
    let current = lease
        .layout
        .relation(&spec.current_relation)
        .ok_or_else(|| SigningDenial::CandidateMismatch)?;
    let mut extra = Vec::new();
    let edges = lease
        .handle()
        .with_runtime(&request, |runtime| {
            observation::adjacency(
                runtime,
                &request,
                lease.snapshot(),
                candidate.read_set.admission.scope_entity_id(),
                current.kind,
                limit,
                &mut extra,
            )
        })
        .map_err(|outcome| SigningDenial::LifecycleObservation(Box::new(outcome)))?;
    if edges.len() >= limit {
        return Err(SigningDenial::Commit(Denial::signing_lifecycle(
            Kind::SigningLifecycleEvidenceCapacity {
                maximum_evidence: limit,
            },
        )));
    }
    candidate
        .read_set
        .append_completed_facts(extra.into_iter(), request.allocation_policy())
        .map_err(SigningDenial::Attempt)?;
    Ok(candidate)
}
