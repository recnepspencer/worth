use crate::authorization::{
    RelationalAuthorizationExactAdjacencyConstraint, RelationalAuthorizationObservationFreshness,
    RelationalAuthorizationObservationPlan,
};
use crate::tests::support::{create_entity, create_relation};

use super::{allow_path, authorization_fixture, forward_traversal, ENTITY_KIND};

#[test]
fn additive_foreign_edge_denies_exact_adjacency_and_stales_prior_observation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut fixture = authorization_fixture(execution);
    let admitted = observe_exact_adjacency(execution, &mut fixture);
    let role = admitted.paths()[0]
        .witness()
        .and_then(|witness| witness.entity_at(1))
        .expect("the baseline must retain its intermediate role");
    let foreign = create_entity(execution, &fixture.runtime, "foreign-exact-target");
    create_relation(
        execution,
        &fixture.runtime,
        role,
        foreign,
        "foreign-exact-edge",
    );

    let current = observe_exact_adjacency(execution, &mut fixture);

    assert!(!current.paths()[0].matched());
    assert_eq!(current.paths()[0].adjacency_lists().len(), 2);
    assert_eq!(current.counters().adjacency_lists_read, 2);
    assert_eq!(current.counters().adjacency_edges_inspected, 3);
    assert_eq!(current.counters().relation_records_inspected, 3);
    let current_snapshot = fixture.runtime.visibility_authority().snapshot();
    assert_eq!(
        fixture
            .runtime
            .compare_authorization_observation(&admitted, current_snapshot, execution)
            .unwrap(),
        RelationalAuthorizationObservationFreshness::Stale
    );
}

#[test]
fn unrelated_graph_population_does_not_expand_exact_adjacency_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut fixture = authorization_fixture(execution);
    let baseline = observe_exact_adjacency(execution, &mut fixture);
    for ordinal in 0..64 {
        let source = create_entity(
            execution,
            &fixture.runtime,
            &format!("unrelated-source-{ordinal}"),
        );
        let target = create_entity(
            execution,
            &fixture.runtime,
            &format!("unrelated-target-{ordinal}"),
        );
        create_relation(
            execution,
            &fixture.runtime,
            source,
            target,
            &format!("unrelated-edge-{ordinal}"),
        );
    }

    let populated = observe_exact_adjacency(execution, &mut fixture);

    assert!(populated.paths()[0].matched());
    assert_eq!(populated.counters(), baseline.counters());
    assert_eq!(populated.counters().adjacency_lists_read, 3);
    assert_eq!(populated.counters().adjacency_edges_inspected, 3);
}

fn observe_exact_adjacency(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    fixture: &mut super::AuthorizationFixture,
) -> crate::authorization::RelationalAuthorizationObservationEvidence {
    let path = allow_path().with_exact_adjacencies([
        RelationalAuthorizationExactAdjacencyConstraint::new(
            1,
            forward_traversal(),
            [fixture.scope],
        ),
    ]);
    let plan = RelationalAuthorizationObservationPlan::try_new(
        fixture.runtime.visibility_authority().snapshot(),
        fixture.principal,
        fixture.scope,
        ENTITY_KIND,
        ENTITY_KIND,
        [path],
        [],
    )
    .expect("the exact adjacency plan is valid");
    fixture
        .runtime
        .observe_authorization(plan, execution)
        .unwrap()
}
