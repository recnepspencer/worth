use super::super::support::*;
use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField, OrderingSelector};
use crate::runtime::{
    WorthQueryReadBuiltInOperator, WorthQueryReadBuiltInOperatorDenialReason,
    WorthQueryReadScopeClass, WorthQueryRuntimeError,
};

#[test]
fn compose_read_executes_operator_owned_successor_walk_detail_as_local_neighborhood() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.successor-walk-detail")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.local_successor_walk_detail(
                "user",
                expanded_manager_schema(),
                manager_relation_name(),
                2,
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("profile", "display_name")
                                .expect("name projection should build"),
                        )
                },
                |shape| {
                    shape
                        .field(
                            AuthoredResultShapeField::new("identity", "id", "id")
                                .expect("identity result-shape field should build"),
                        )
                        .field(
                            AuthoredResultShapeField::new(
                                "profile",
                                "display_name",
                                "display_name",
                            )
                            .expect("name result-shape field should build"),
                        )
                },
            )
        })
        .expect("operator-owned successor walk detail should execute");

    assert!(!result.rows().is_empty());
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::LocalNeighborhood
    );
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::SuccessorWalk]
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &bounded_ancestor_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_executes_operator_owned_successor_walk_collection_as_local_neighborhood() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.successor-walk-collection")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.local_successor_walk_collection(
                "user",
                expanded_manager_schema(),
                manager_relation_name(),
                2,
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("profile", "display_name")
                                .expect("name projection should build"),
                        )
                        .order_by(
                            OrderingSelector::ascending("profile", "display_name")
                                .expect("ordering should build"),
                        )
                },
                |shape| {
                    shape
                        .field(
                            AuthoredResultShapeField::new("identity", "id", "id")
                                .expect("identity result-shape field should build"),
                        )
                        .field(
                            AuthoredResultShapeField::new(
                                "profile",
                                "display_name",
                                "display_name",
                            )
                            .expect("name result-shape field should build"),
                        )
                },
            )
        })
        .expect("operator-owned successor walk collection should execute");

    assert_collection_receipt(&result, WorthQueryReadScopeClass::LocalNeighborhood);
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::SuccessorWalk]
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &bounded_ancestor_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_denies_zero_depth_successor_walk() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.successor-walk-zero-depth")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.local_successor_walk_detail(
                "user",
                expanded_manager_schema(),
                manager_relation_name(),
                0,
                |query| {
                    query.project(
                        AspectFieldSelector::new("identity", "id")
                            .expect("identity projection should build"),
                    )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "id")
                            .expect("identity result-shape field should build"),
                    )
                },
            )
        })
        .expect_err("zero-depth successor walks should deny");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::SuccessorWalk,
                WorthQueryReadBuiltInOperatorDenialReason::ZeroDepth,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_degenerate_one_hop_successor_walk() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.successor-walk-degenerate")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.local_successor_walk_detail(
                "user",
                expanded_manager_schema(),
                manager_relation_name(),
                1,
                |query| {
                    query.project(
                        AspectFieldSelector::new("identity", "id")
                            .expect("identity projection should build"),
                    )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "id")
                            .expect("identity result-shape field should build"),
                    )
                },
            )
        })
        .expect_err("one-hop successor walks should deny in favor of direct edge");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::SuccessorWalk,
                WorthQueryReadBuiltInOperatorDenialReason::DegenerateSuccessorWalkShape,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}
