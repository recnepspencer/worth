use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationAdjacencyDirection as DecisionDirection;
use worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial;
use worth_relational::facade::runtime::RelationalAdjacencyDirection as NativeDirection;

// The native revision is one selected owner-index lookup; it does not walk
// the adjacency population. It covers pair presence and ABA.
const NATIVE_REVISION_LOOKUP_WORK: usize = 1;

pub fn rebase_decision_adjacency(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    fact: Fact,
) -> Fact {
    let endpoints = prepare_endpoints(&fact, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap();
    prepare_decision_adjacency(execution, runtime, snapshot, &fact, endpoints)
        .expect("fixture request remains live")
        .unwrap_or(fact)
}

pub(super) fn prepare_decision_adjacency(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    fact: &Fact,
    prepared_endpoints: Option<
        crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints,
    >,
) -> Result<Option<Fact>, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    let (relation_kind, anchor, direction, limit) = match fact {
        Fact::Relation {
            relation_kind,
            from,
            ..
        } => (
            *relation_kind,
            *from,
            NativeDirection::Outgoing,
            NATIVE_REVISION_LOOKUP_WORK,
        ),
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            maximum_work_units,
            ..
        } => {
            let native_direction = match direction {
                DecisionDirection::Outgoing => NativeDirection::Outgoing,
                DecisionDirection::Incoming => NativeDirection::Incoming,
            };
            (
                *relation_kind,
                *anchor,
                native_direction,
                *maximum_work_units,
            )
        }
        _ => return Ok(None),
    };
    let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
        return Ok(None);
    };
    let native_revision = match view.bounded_adjacency_structural_revision(
        execution,
        anchor,
        relation_kind,
        direction,
        limit,
    ) {
        Ok(revision) => revision.revision(),
        Err(AdjacencyStructuralRevisionDenial::Execution(cause)) => return Err(cause),
        // A zero recorded limit, or a missing anchor or basis, leaves the
        // decision fact without a native revision.
        Err(
            AdjacencyStructuralRevisionDenial::WorkBudgetExceeded
            | AdjacencyStructuralRevisionDenial::AnchorUnavailable
            | AdjacencyStructuralRevisionDenial::BasisUnavailable,
        ) => return Ok(None),
    };
    let Some(endpoints) = prepared_endpoints else {
        return Ok(None);
    };
    Ok(Some(Fact::SourceAdjacencyRevision {
        relation_kind,
        anchor,
        direction,
        native_revision,
        comparison_work_limit: limit,
        endpoints,
    }))
}

/// BEFORE effects: every endpoint is already known from the complete decision
/// fact. The later selected native revision changes no endpoint payload.
pub(super) fn prepare_endpoints(
    fact: &Fact,
    control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
) -> Result<Option<crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints>, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>{
    use crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints as Endpoints;
    let endpoints = match fact {
        Fact::Relation { to, .. } => {
            Endpoints::from_exact_iterator(1, std::iter::once(*to), control)?
        }
        Fact::Adjacency {
            direction,
            relations,
            ..
        } => Endpoints::from_exact_iterator(
            relations.len(),
            relations.iter().map(|relation| match direction {
                DecisionDirection::Outgoing => relation.to,
                DecisionDirection::Incoming => relation.from,
            }),
            control,
        )?,
        _ => return Ok(None),
    };
    Ok(Some(endpoints))
}
