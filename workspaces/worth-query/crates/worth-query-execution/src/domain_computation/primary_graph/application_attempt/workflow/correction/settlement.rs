//! Exact canonical settlement admission for an occurrence's current continuation.
use super::{
    unavailable, WorkflowOperationOrigin, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, workflow_instance_observation::settlement::observe_transition_state,
    WorthQueryApplicationAdjacencyDirection,
};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

pub(super) fn require_settled_origin_and_custody(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    origin: &WorkflowOperationOrigin,
    custody: Option<&str>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let settled = operation_settlement(execution, runtime, snapshot, origin, origin.transition())?;
    match settled {
        Some(receipt) if receipt == origin.operation_receipt_identity() => {}
        None if custody == Some(origin.transition()) => return Err(pending()),
        _ => {
            return Err(unavailable(
                "original workflow operation settlement is unavailable or changed",
            ))
        }
    }
    if let Some(custody) = custody
        && custody != origin.transition()
        && operation_settlement(execution, runtime, snapshot, origin, custody)?.is_none()
    {
        return Err(pending());
    }
    Ok(())
}

fn pending() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
        "correction waits for the original workflow owner to settle external custody",
    )
}

fn operation_settlement(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    origin: &WorkflowOperationOrigin,
    identity: &str,
) -> Result<Option<[u8; 32]>, WorthQueryApplicationAttemptDenial> {
    let layout = origin.layout();
    // Two candidates distinguish the required singleton from ambiguity.
    let denied = |cause| {
        WorthQueryApplicationAttemptDenial::indexed_lookup_denied(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "workflow correction settlement index",
            cause,
        )
    };
    let lookup = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        layout.transition.identity_index_id,
        layout.transition.entity_kind,
        layout.transition.identity.clone(),
        AspectValue::String(InternedString::Raw(identity.to_owned())),
        2,
    )
    .map_err(denied)?;
    let outcome = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(lookup, BoundedIndexParityMode::Production, execution)
        .map_err(denied)?;
    if outcome.overflowed() {
        return Err(unavailable(
            "workflow correction settlement identity is ambiguous",
        ));
    }
    let candidates = outcome.into_candidate_entity_ids();
    let transition = match candidates.as_slice() {
        [] => return Ok(None),
        [transition] => *transition,
        _ => {
            return Err(unavailable(
                "workflow correction settlement identity is ambiguous",
            ))
        }
    };
    let membership = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.instance_transition_relation,
        transition,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
    )?
    .ok_or_else(|| unavailable("workflow correction settlement membership is unavailable"))?;
    if membership.len() != 1 || membership[0].from != origin.instance() {
        return Err(unavailable(
            "workflow correction settlement belongs to a different instance",
        ));
    }
    let settled = observe_transition_state(
        execution,
        runtime,
        snapshot,
        layout,
        transition,
        &mut Vec::new(),
    )?;
    settled
        .operation_receipt_identity()
        .map(Some)
        .ok_or_else(|| unavailable("workflow correction settlement is not a performed operation"))
}
