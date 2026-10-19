//! Native bounded inverse traversal retains the exact preimage used to route a source edit.
use super::{
    attempt, capacity, resource, stopped, unavailable, Batch, EntityId, Fact, KindId, Lease,
    Outcome, PathDirection, QueryRequestExecution,
};
use crate::domain_computation::primary_graph::application_attempt::{
    fact::WorthQueryApplicationObservedRelation as Edge,
    fact::{observe_adjacency_with_work, AdjacencyObservationDenial},
    WorthQueryApplicationAdjacencyDirection as Direction,
};
use crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation;
pub(super) fn edges(
    lease: &Lease,
    anchor: EntityId,
    kind: KindId,
    direction: PathDirection,
    execution: &QueryRequestExecution<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batch: &mut Batch,
) -> Result<Vec<Edge>, Outcome> {
    let direction = match direction {
        PathDirection::Forward => Direction::Outgoing,
        PathDirection::Reverse => Direction::Incoming,
    };
    for fact in &batch.facts {
        context.checkpoint(1).map_err(stopped)?;
        if let Fact::Adjacency {
            relation_kind,
            anchor: observed,
            direction: observed_direction,
            relations,
            ..
        } = fact
            && *relation_kind == kind
            && *observed == anchor
            && *observed_direction == direction
        {
            reserve(relations.len(), execution, &mut batch.memory)?;
            return Ok(relations.clone());
        }
    }
    let maximum = usize::try_from(context.remaining_work())
        .unwrap_or(usize::MAX - 1)
        .min(usize::MAX - 1);
    let (edges, work) = lease.handle().with_runtime(request, |runtime| {
        observe_adjacency_with_work(runtime, request, lease.snapshot(), kind, anchor, direction, maximum)
    }).map_err(|cause| match cause {
        AdjacencyObservationDenial::Execution(cause) => attempt(super::super::super::super::WorthQueryApplicationAttemptDenial::from(cause)),
        AdjacencyObservationDenial::SnapshotUnavailable => unavailable(),
        AdjacencyObservationDenial::WorkBudgetExceeded => capacity(),
        AdjacencyObservationDenial::Interrupted(event) => attempt(super::super::super::super::WorthQueryApplicationAttemptDenial::new(
            match event.interruption() {
                worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => super::super::super::super::WorthQueryApplicationAttemptDenialKind::Cancelled,
                worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => super::super::super::super::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded,
            }, "signing subject path",
        )),
    })?;
    context
        .checkpoint(u64::try_from(work).map_err(|_| capacity())?)
        .map_err(stopped)?;
    reserve(edges.len(), execution, &mut batch.memory)?;
    batch.facts.push(Fact::Adjacency {
        relation_kind: kind,
        anchor,
        direction,
        maximum_work_units: work,
        relations: edges.clone(),
    });
    Ok(edges)
}
fn reserve(
    count: usize,
    execution: &QueryRequestExecution<'_>,
    memory: &mut Vec<QueryMemoryReservation>,
) -> Result<(), Outcome> {
    // Both retained adjacency and transient endpoint sets/copies remain funded until publication.
    let bytes = count
        .checked_add(1)
        .and_then(|n| n.checked_mul(size_of::<Edge>() * 8 + size_of::<Fact>()))
        .ok_or_else(capacity)?;
    memory.push(
        execution
            .reserve(u64::try_from(bytes).map_err(|_| capacity())?)
            .map_err(resource)?,
    );
    Ok(())
}
