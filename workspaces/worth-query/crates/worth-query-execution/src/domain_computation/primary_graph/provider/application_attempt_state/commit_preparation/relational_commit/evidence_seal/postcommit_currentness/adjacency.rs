use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as DecisionDirection,
    WorthQueryApplicationObservedFact as Fact,
};
use worth_relational::facade::runtime::{
    AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection as NativeDirection,
};

// The native revision is one selected owner-index lookup; it does not walk
// the adjacency population. It covers pair presence and ABA.
const NATIVE_REVISION_LOOKUP_WORK: usize = 1;

#[cfg(test)]
pub(super) fn rebase_decision_adjacency(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    fact: Fact,
    admitted_work: Option<usize>,
) -> Fact {
    let endpoints = prepare_endpoints(&fact, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap();
    prepare_decision_adjacency(
        execution,
        runtime,
        snapshot,
        &fact,
        admitted_work,
        endpoints,
    )
    .expect("fixture request remains live")
    .unwrap_or(fact)
}

pub(super) fn prepare_decision_adjacency(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    fact: &Fact,
    admitted_work: Option<usize>,
    prepared_endpoints: Option<
        crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints,
    >,
) -> Result<Option<Fact>, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    if admitted_work == Some(0) {
        return Ok(None);
    }
    let (relation_kind, anchor, direction, limit) = match fact {
        Fact::Relation {
            relation_kind,
            from,
            ..
        } => (
            *relation_kind,
            *from,
            NativeDirection::Outgoing,
            admitted_work.map_or(NATIVE_REVISION_LOOKUP_WORK, |maximum| {
                NATIVE_REVISION_LOOKUP_WORK.min(maximum)
            }),
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
                admitted_work.map_or(*maximum_work_units, |maximum| {
                    (*maximum_work_units).min(maximum)
                }),
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
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, live_scope, AccountOwner, PrincipalIdentityField,
    };
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

    fn admission(work: u64) -> crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission{
        crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: Some(work),
                maximum_preparation_bytes: None,
            },
        )
    }

    #[test]
    fn pair_rebase_requires_admitted_native_work_and_never_retains_partial_facts() {
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
            let mut zero_work = admission(0);
            assert!(matches!(
                rebase_decision_adjacency(&host_custody.borrowed_request_for_test(), runtime, snapshot, fact.clone(), Some(64)),
                Fact::SourceAdjacencyRevision {
                    comparison_work_limit: 64,
                    native_revision: Some(_),
                    ..
                }
            ));
            assert!(
                matches!(
                    super::super::rebase(&host_custody.borrowed_request_for_test(),
                        runtime,
                        snapshot,
                        super::super::PreparedSourceFactRebase::admit(vec![fact], [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(),
                        &std::collections::BTreeSet::new(),
                        true,
                        Some(0),
                        &mut zero_work
                    ),
                    super::super::RebasedSourceFacts::VerificationRequired { .. }
                ),
                "failed relation revision acquisition marks the entire output non-reusable"
            );
            let stale_revision = Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor: principal.entity_id(),
                direction: NativeDirection::Outgoing,
                native_revision: None,
                comparison_work_limit: 0,
                endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
            };
            let mut producer_work = admission(64);
            assert!(matches!(
                super::super::rebase(&host_custody.borrowed_request_for_test(),
                    runtime,
                    snapshot,
                    super::super::PreparedSourceFactRebase::admit(vec![stale_revision.clone()], [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None))
                        .unwrap(),
                    &std::collections::BTreeSet::new(),
                    true,
                    Some(64),
                    &mut producer_work,
                ),
                super::super::RebasedSourceFacts::VerificationRequired { .. }
            ));
            let mut ordinary_work = admission(64);
            match super::super::rebase(&host_custody.borrowed_request_for_test(),
                runtime,
                snapshot,
                super::super::PreparedSourceFactRebase::admit(vec![stale_revision.clone()], [].into(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None))
                    .unwrap(),
                &std::collections::BTreeSet::new(),
                false,
                Some(64),
                &mut ordinary_work,
            ) {
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
