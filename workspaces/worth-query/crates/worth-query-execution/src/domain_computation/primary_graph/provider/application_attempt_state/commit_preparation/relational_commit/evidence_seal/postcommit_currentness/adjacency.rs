use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection as DecisionDirection;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use worth_relational::facade::runtime::AdjacencyStructuralRevisionDenial;
use worth_relational::facade::runtime::RelationalAdjacencyDirection as NativeDirection;

// The native revision is one selected owner-index lookup; it does not walk
// the adjacency population. It covers pair presence and ABA.
const NATIVE_REVISION_LOOKUP_WORK: usize = 1;

#[cfg(test)]
pub(super) fn rebase_decision_adjacency(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
    use crate::domain_computation::primary_graph::tests::fixture::live_scope;
    use crate::domain_computation::primary_graph::tests::fixture::AccountOwner;
    use crate::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

    #[test]
    fn pair_rebase_uses_native_revision_and_never_retains_partial_facts() {
        let host_request = crate::test_request_policy::serial_request();
        let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let world = installed_authorization_world(true);
        let graph = world.application.runtime.primary_graph().unwrap();
        let relation_kind = graph
            .layout
            .relation(AccountOwner::reference().name())
            .unwrap()
            .kind;
        let selected = world
            .application
            .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
            .expect("the fixture read owns its installed host call");
        let principal = world
            .application
            .with_host_advancement(|phase| {
                selected
                    .resolve_entity(
                        &phase,
                        PrincipalIdentityField::reference(),
                        1,
                        &live_scope(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let fact = Fact::Relation {
            relation_kind,
            from: principal.entity_id(),
            to: principal.entity_id(),
            matching_relations: Vec::new(),
        };
        world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); graph.integration_handle().with_runtime(request_execution, |runtime| {
            let snapshot = selected.application_basis().snapshot_handle();

            assert!(matches!(
                rebase_decision_adjacency(&host_custody.borrowed_request_for_test(), runtime, snapshot, fact.clone()),
                Fact::SourceAdjacencyRevision {
                    comparison_work_limit: 1,
                    native_revision: Some(_),
                    ..
                }
            ));
            assert!(
                matches!(
                    super::super::rebase(&host_custody.borrowed_request_for_test(), runtime, snapshot, super::super::PreparedSourceFactRebase::admit(vec![fact], [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(), &std::collections::BTreeSet::new(), true),
                    super::super::RebasedSourceFacts::Exact(_)
                ),
                "a complete pair fact rebases through its native structural revision"
            );
            let stale_revision = Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor: principal.entity_id(),
                direction: NativeDirection::Outgoing,
                native_revision: None,
                comparison_work_limit: 0,
                endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
            };

            assert!(matches!(
                super::super::rebase(&host_custody.borrowed_request_for_test(), runtime, snapshot, super::super::PreparedSourceFactRebase::admit(vec![stale_revision.clone()], [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None))
                        .unwrap(), &std::collections::BTreeSet::new(), true),
                super::super::RebasedSourceFacts::VerificationRequired { .. }
            ));

            match super::super::rebase(&host_custody.borrowed_request_for_test(), runtime, snapshot, super::super::PreparedSourceFactRebase::admit(vec![stale_revision.clone()], [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None))
                    .unwrap(), &std::collections::BTreeSet::new(), false) {
                super::super::RebasedSourceFacts::Exact(facts) => {
                    assert_eq!(facts.iter().cloned().collect::<Vec<_>>(), &[stale_revision])
                }
                other => {
                    panic!("non-output evidence must retain its comparison posture: {other:?}")
                }
            }
        }) }).expect("the fixture read owns its installed host call");
    }
}
