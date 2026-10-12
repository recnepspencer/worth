use super::super::support::*;
use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField, OrderingSelector};
use crate::runtime::{
    WorthQueryReadBuiltInOperator, WorthQueryReadBuiltInOperatorDenialReason,
    WorthQueryReadScopeClass, WorthQueryRuntimeError,
};

#[test]
fn compose_read_executes_operator_owned_bounded_descendant_detail() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-descendant-detail")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.anchored_bounded_descendant_detail(
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
        .expect("operator-owned bounded descendant detail should execute");

    assert!(!result.rows().is_empty());
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::AnchoredExpansion
    );
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::BoundedDescendant]
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &bounded_descendant_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_executes_operator_owned_bounded_descendant_collection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-descendant-collection")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.anchored_bounded_descendant_collection(
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
        .expect("operator-owned bounded descendant collection should execute");

    assert!(!result.rows().is_empty());
    assert_collection_receipt(&result, WorthQueryReadScopeClass::AnchoredExpansion);
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::BoundedDescendant]
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &bounded_descendant_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_denies_zero_depth_bounded_ancestor_walk() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-ancestor-zero-depth")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.anchored_bounded_ancestor_detail(
                "user",
                manager_schema(),
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
        .expect_err("bounded ancestor should deny zero depth");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::BoundedAncestor,
                WorthQueryReadBuiltInOperatorDenialReason::ZeroDepth,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_zero_depth_bounded_descendant_walk() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-descendant-zero-depth")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.anchored_bounded_descendant_detail(
                "user",
                manager_schema(),
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
        .expect_err("bounded descendant should deny zero depth");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::BoundedDescendant,
                WorthQueryReadBuiltInOperatorDenialReason::ZeroDepth,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_one_hop_bounded_descendant_walk() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-descendant-one-hop")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.anchored_bounded_descendant_detail(
                "user",
                manager_schema(),
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
        .expect_err("bounded descendant should deny one-hop shapes");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::BoundedDescendant,
                WorthQueryReadBuiltInOperatorDenialReason::DegenerateBoundedWalkShape,
            );
            assert!(denial.message().contains("use direct edge"));
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_one_hop_bounded_ancestor_walk() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-ancestor-one-hop")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.anchored_bounded_ancestor_detail(
                "user",
                manager_schema(),
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
        .expect_err("bounded ancestor should deny one-hop shapes");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::BoundedAncestor,
                WorthQueryReadBuiltInOperatorDenialReason::DegenerateBoundedWalkShape,
            );
            assert!(denial.message().contains("use direct edge"));
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}
