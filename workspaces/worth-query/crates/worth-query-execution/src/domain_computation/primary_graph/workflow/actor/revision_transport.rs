//! Convert only native observations at their original immutable basis.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAdjacencyDirection as Direction,
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationObservedFact as Fact, WorthQueryApplicationSourceAdjacencyEndpoints,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{RelationalAdjacencyDirection, RelationalRuntime},
    snapshots::SnapshotHandle,
};

pub(super) fn capture(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &super::super::schema::WorthQueryWorkflowLayout,
    fact: &Fact,
) -> Result<Fact, Denial> {
    let invalid = || {
        Denial::new(
            Kind::InvalidAuthoritativeValue,
            "workflow actor native dependency unavailable",
        )
    };
    // Compare before conversion. Revision capture may never bless a fact from
    // another basis or a caller's value as a freshly observed native fact.
    if !fact.remains_equal_in(runtime, execution, snapshot)? {
        return Err(invalid());
    }
    Ok(match fact {
        Fact::Field { .. } | Fact::AbsentField { .. } => fact.clone(),
        Fact::Relation {
            relation_kind,
            from,
            ..
        } => adjacency(
            execution,
            runtime,
            snapshot,
            *relation_kind,
            *from,
            RelationalAdjacencyDirection::Outgoing,
        )?,
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            ..
        } => adjacency(
            execution,
            runtime,
            snapshot,
            *relation_kind,
            *anchor,
            match direction {
                Direction::Outgoing => RelationalAdjacencyDirection::Outgoing,
                Direction::Incoming => RelationalAdjacencyDirection::Incoming,
            },
        )?,
        Fact::WorkflowHistoryBasis { instance, .. } => adjacency(
            execution,
            runtime,
            snapshot,
            layout.instance_transition_relation,
            *instance,
            RelationalAdjacencyDirection::Outgoing,
        )?,
        Fact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            ..
        } => adjacency(
            execution,
            runtime,
            snapshot,
            *relation_kind,
            *anchor,
            *direction,
        )?,
        Fact::Entity { .. }
        | Fact::SourceEntity { .. }
        | Fact::SourceAspectRevision { .. }
        | Fact::SourceFieldRevision { .. } => fact.clone(),
        _ => return Err(invalid()),
    })
}
fn adjacency(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: RelationalAdjacencyDirection,
) -> Result<Fact, Denial> {
    use worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial as Stop;
    let denied = || {
        Denial::new(
            Kind::InvalidAuthoritativeValue,
            "workflow actor membership revision unavailable",
        )
    };
    let revision = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(denied)?
        .bounded_adjacency_structural_revision(execution, anchor, relation_kind, direction, 1)
        .map_err(|cause| match cause {
            Stop::Execution(cause) => {
                Denial::relational_read_stopped(cause, "workflow actor membership")
            }
            Stop::WorkBudgetExceeded | Stop::AnchorUnavailable | Stop::BasisUnavailable => denied(),
        })?
        .revision();
    // This is the same revision-only native fact used by workflow progress.
    // It claims no enumerated endpoint payload and compares structural stamps.
    Ok(Fact::SourceAdjacencyRevision {
        relation_kind,
        anchor,
        direction,
        native_revision: revision,
        comparison_work_limit: 1,
        endpoints: WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
    })
}
