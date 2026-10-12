use crate::authorization::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationObservationFreshness,
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathPlan,
};
use crate::facade::history::BranchId;
use crate::tests::support::{
    create_entity, create_relation, delete_relation_on_branch, runtime_with_test_schema,
};

use super::{forward_traversal, ENTITY_KIND};

#[test]
fn exact_next_hop_anchor_ignores_same_source_population() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let principal = create_entity(execution, &runtime, "anchored-principal");
    let anchor = create_entity(execution, &runtime, "anchored-role");
    let scope = create_entity(execution, &runtime, "anchored-scope");
    create_relation(
        execution,
        &runtime,
        principal,
        anchor,
        "anchored-principal-role",
    );
    create_relation(execution, &runtime, anchor, scope, "anchored-role-scope");

    let baseline = observe_anchored_path(execution, &runtime, principal, anchor, scope);
    for ordinal in 0..64 {
        let unrelated = create_entity(execution, &runtime, &format!("same-source-grant-{ordinal}"));
        create_relation(
            execution,
            &runtime,
            principal,
            unrelated,
            &format!("same-source-edge-{ordinal}"),
        );
    }
    let populated = observe_anchored_path(execution, &runtime, principal, anchor, scope);

    assert_eq!(populated.counters(), baseline.counters());
    assert_eq!(
        populated.paths()[0].witness(),
        baseline.paths()[0].witness()
    );
    assert_eq!(populated.counters().adjacency_edges_inspected, 2);
}

#[test]
fn exact_next_hop_anchor_retains_revocation_currentness() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let principal = create_entity(execution, &runtime, "revoked-anchor-principal");
    let anchor = create_entity(execution, &runtime, "revoked-anchor-role");
    let scope = create_entity(execution, &runtime, "revoked-anchor-scope");
    let exact = create_relation(
        execution,
        &runtime,
        principal,
        anchor,
        "revoked-principal-role",
    );
    create_relation(execution, &runtime, anchor, scope, "revoked-role-scope");
    let admitted = observe_anchored_path(execution, &runtime, principal, anchor, scope);

    let revoked =
        delete_relation_on_branch(execution, &runtime, exact, BranchId("main".to_owned()));

    assert_eq!(
        runtime
            .compare_authorization_observation(&admitted, revoked.snapshot.clone(), execution)
            .unwrap(),
        RelationalAuthorizationObservationFreshness::Stale,
    );
}

fn observe_anchored_path(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    principal: crate::identity::data::EntityId,
    anchor: crate::identity::data::EntityId,
    scope: crate::identity::data::EntityId,
) -> crate::authorization::RelationalAuthorizationObservationEvidence {
    let path = RelationalAuthorizationPathPlan::new([forward_traversal(), forward_traversal()], [])
        .with_entity_anchors([RelationalAuthorizationEntityAnchor::new(
            1,
            ENTITY_KIND,
            anchor,
        )]);
    let plan = RelationalAuthorizationObservationPlan::try_new(
        runtime.visibility_authority().snapshot(),
        principal,
        scope,
        ENTITY_KIND,
        ENTITY_KIND,
        [path],
        [],
    )
    .unwrap();
    runtime.observe_authorization(plan, execution).unwrap()
}
