//! Native bounded inverse traversal retains the exact preimage used to route a source edit.
use {crate::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedRelation as Edge, crate::domain_computation::primary_graph::application_attempt::fact::observe_adjacency_with_work, crate::domain_computation::primary_graph::application_attempt::fact::AdjacencyObservationDenial, crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationAdjacencyDirection as Direction};
use {
    super::attempt, super::capacity, super::stopped, super::unavailable, super::Batch,
    super::EntityId, super::Fact, super::KindId, super::Lease, super::Outcome,
    super::PathDirection, crate::domain_computation::request_execution::QueryRequestExecution,
};
pub(super) fn edges(
    lease: &Lease,
    anchor: EntityId,
    kind: KindId,
    direction: PathDirection,
    _execution: &QueryRequestExecution<'_>,
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
            return Ok(relations.clone());
        }
    }
    let maximum = context
        .remaining_work()
        .map(|work| usize::try_from(work).map_err(|_| capacity()))
        .transpose()?;
    let (edges, work) = lease.handle().with_runtime(request, |runtime| {
        observe_adjacency_with_work(runtime, request, lease.snapshot(), kind, anchor, direction, maximum)
    }).map_err(|cause| match cause {
        AdjacencyObservationDenial::Execution(cause) => attempt(super::super::super::super::WorthQueryApplicationAttemptDenial::from(cause)),
        AdjacencyObservationDenial::SnapshotUnavailable => unavailable(),
        AdjacencyObservationDenial::WorkBudgetExceeded => capacity(),
        AdjacencyObservationDenial::Interrupted(event) => attempt(super::super::super::super::WorthQueryApplicationAttemptDenial::new(
            match event.interruption() {
                worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind::Cancelled,
                worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind::DeadlineExceeded,
            }, "signing subject path",
        )),
    })?;
    context
        .checkpoint(u64::try_from(work).map_err(|_| capacity())?)
        .map_err(stopped)?;
    batch.facts.push(Fact::Adjacency {
        relation_kind: kind,
        anchor,
        direction,
        maximum_work_units: work,
        relations: edges.clone(),
    });
    Ok(edges)
}
