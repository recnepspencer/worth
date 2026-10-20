use worth_relational::facade::identity::{EntityId, KindId};

use super::{WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationObservedRelation};

pub(super) fn remains_equal(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: usize,
    expected: &[WorthQueryApplicationObservedRelation],
) -> Result<bool, super::super::WorthQueryApplicationAttemptDenial> {
    Ok(observe_adjacency(
        runtime,
        execution,
        snapshot,
        relation_kind,
        anchor,
        direction,
        maximum_work_units,
    )?
    .is_some_and(|current| current == expected))
}

pub(in crate::domain_computation::primary_graph) fn observe_adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: usize,
) -> Result<
    Option<Vec<WorthQueryApplicationObservedRelation>>,
    super::super::WorthQueryApplicationAttemptDenial,
> {
    let observed = observe_adjacency_checked(
        runtime,
        execution,
        snapshot,
        relation_kind,
        anchor,
        direction,
        maximum_work_units,
    );
    resolve_observation(observed)
}

/// Historical identity observation only. Ordinary adjacency remains Live-only.
pub(in crate::domain_computation::primary_graph) fn observe_audit_outgoing_adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    maximum_work_units: usize,
) -> Result<
    Option<Vec<WorthQueryApplicationObservedRelation>>,
    super::super::WorthQueryApplicationAttemptDenial,
> {
    resolve_observation(
        observe_with_posture(
            runtime,
            execution,
            snapshot,
            relation_kind,
            anchor,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            Some(maximum_work_units),
            RelationPosture::IncludingAudit,
        )
        .map(|(relations, _)| relations),
    )
}

fn resolve_observation(
    observed: Result<Vec<WorthQueryApplicationObservedRelation>, AdjacencyObservationDenial>,
) -> Result<
    Option<Vec<WorthQueryApplicationObservedRelation>>,
    super::super::WorthQueryApplicationAttemptDenial,
> {
    match observed {
        Ok(relations) => Ok(Some(relations)),
        Err(AdjacencyObservationDenial::Execution(cause)) => Err(
            super::super::WorthQueryApplicationAttemptDenial::relational_read_stopped(
                cause,
                "adjacency observation",
            ),
        ),
        Err(AdjacencyObservationDenial::Interrupted(event)) => {
            Err(super::super::WorthQueryApplicationAttemptDenial::new(
                match event.interruption() {
                    worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
                        super::super::WorthQueryApplicationAttemptDenialKind::Cancelled
                    }
                    worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => {
                        super::super::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
                    }
                },
                "adjacency observation",
            ))
        }
        Err(
            AdjacencyObservationDenial::SnapshotUnavailable
            | AdjacencyObservationDenial::WorkBudgetExceeded,
        ) => Ok(None),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum AdjacencyObservationDenial {
    SnapshotUnavailable,
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Interrupted(worth_relational::facade::mvcc::RelationalInterruptionEvent),
    WorkBudgetExceeded,
}

pub(in crate::domain_computation::primary_graph) fn observe_adjacency_checked(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: usize,
) -> Result<Vec<WorthQueryApplicationObservedRelation>, AdjacencyObservationDenial> {
    observe_adjacency_with_work(
        runtime,
        execution,
        snapshot,
        relation_kind,
        anchor,
        direction,
        Some(maximum_work_units),
    )
    .map(|(relations, _)| relations)
}

pub(in crate::domain_computation::primary_graph) fn observe_adjacency_with_work(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: Option<usize>,
) -> Result<(Vec<WorthQueryApplicationObservedRelation>, usize), AdjacencyObservationDenial> {
    observe_with_posture(
        runtime,
        execution,
        snapshot,
        relation_kind,
        anchor,
        direction,
        maximum_work_units,
        RelationPosture::Live,
    )
}

enum RelationPosture {
    Live,
    IncludingAudit,
}

fn observe_with_posture(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: Option<usize>,
    posture: RelationPosture,
) -> Result<(Vec<WorthQueryApplicationObservedRelation>, usize), AdjacencyObservationDenial> {
    if let Err(stop) = execution.check_live() {
        return Err(match worth_relational::facade::mvcc::RelationalInterruptionEvent::from_request_stop(
            stop, worth_relational::facade::mvcc::RelationalInterruptionBoundary::ObservationAdmission,
        ) {
            Ok(event) => AdjacencyObservationDenial::Interrupted(event),
            Err(stop) => AdjacencyObservationDenial::Execution(stop.into()),
        });
    }
    execution
        .consult()
        .map_err(|stop| AdjacencyObservationDenial::Execution(stop.into()))?;
    let pin = runtime.read_truth().pin_read_capability(snapshot, &[], execution)
        .map_err(|refusal| match refusal {
            worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(stop) => AdjacencyObservationDenial::Execution(stop.into()),
            worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::SnapshotUnavailable
            | worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::IndexNotInstalled(_) => AdjacencyObservationDenial::SnapshotUnavailable,
        })?;
    let frontier = std::collections::BTreeSet::from([anchor]);
    let work = maximum_work_units
        .map(|maximum| {
            maximum
                .checked_add(1)
                .ok_or(AdjacencyObservationDenial::WorkBudgetExceeded)
        })
        .transpose()?;
    let read = match (direction, posture) {
        (WorthQueryApplicationAdjacencyDirection::Outgoing, RelationPosture::IncludingAudit) => pin
            .bounded_outgoing_relations_including_audit_for_frontier(
                execution,
                &frontier,
                relation_kind,
                work,
            ),
        (WorthQueryApplicationAdjacencyDirection::Outgoing, RelationPosture::Live) => {
            pin.bounded_outgoing_relations_for_frontier(execution, &frontier, relation_kind, work)
        }
        (WorthQueryApplicationAdjacencyDirection::Incoming, RelationPosture::Live) => {
            pin.bounded_incoming_relations_for_frontier(execution, &frontier, relation_kind, work)
        }
        (WorthQueryApplicationAdjacencyDirection::Incoming, RelationPosture::IncludingAudit) => {
            unreachable!("audit observation is outgoing")
        }
    }
    .map_err(|stop| match stop {
        worth_relational::facade::runtime::BoundedFrontierReadStop::Admission(stop) => {
            AdjacencyObservationDenial::Execution(stop.into())
        }
        worth_relational::facade::runtime::BoundedFrontierReadStop::Limit(_) => {
            AdjacencyObservationDenial::WorkBudgetExceeded
        }
    })?;
    let work_units = read.work_units();
    let relations = read
        .into_records()
        .into_iter()
        .map(|record| WorthQueryApplicationObservedRelation {
            relation_id: record.relation_id,
            from: record.source,
            to: record.target,
        })
        .collect();
    Ok((relations, work_units))
}
