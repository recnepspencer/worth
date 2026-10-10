use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, TraversalSelector,
    WorthQueryPredicateOperand,
};
use crate::runtime::{
    WorthQueryReadBuiltInOperator, WorthQueryReadDenialKind, WorthQueryReadExecutionEngine,
    WorthQueryReadOperatorFamily, WorthQueryReadScopeClass, WorthQueryRuntimeError,
};

#[test]
fn compose_read_executes_local_traversal_detail_with_attached_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.traversal")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.local_detail(
                "user",
                manager_schema(),
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
                        .traverse(
                            TraversalSelector::bounded("manager", 1)
                                .expect("bounded traversal should build"),
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
        .expect("local traversal detail read should execute");

    assert!(!result.rows().is_empty());
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::LocalNeighborhood
    );
    assert_eq!(
        result.receipt().execution_engine(),
        &WorthQueryReadExecutionEngine::QueryRuntimeCurrent
    );
    assert!(result
        .receipt()
        .operator_families()
        .contains(&WorthQueryReadOperatorFamily::Projection));
    assert!(result
        .receipt()
        .operator_families()
        .contains(&WorthQueryReadOperatorFamily::Traversal));
    assert!(result.receipt().built_in_operator_coverage().is_empty());
    assert_eq!(
        result.receipt().breadth().planned_traversal_clause_count(),
        1
    );
    assert_eq!(
        result.receipt().breadth().planned_traversal_depth_limit(),
        1
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &direct_edge_synthetic_runtime_surfaces(),
    );
    assert!(!result.receipt().read_graph_digest().is_empty());
    assert!(!result.receipt().result_digest().is_empty());
}

#[test]
fn compose_read_executes_operator_owned_direct_edge_detail() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-direct-edge-detail")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.local_direct_edge_detail(
                "user",
                manager_schema(),
                manager_relation_name(),
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
        .expect("operator-owned direct-edge detail should execute");

    assert!(!result.rows().is_empty());
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::LocalNeighborhood
    );
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::DirectEdge]
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &direct_edge_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_executes_operator_owned_bounded_ancestor_detail() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-bounded-ancestor-detail")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.anchored_bounded_ancestor_detail(
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
        .expect("operator-owned bounded-ancestor detail should execute");

    assert!(!result.rows().is_empty());
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::AnchoredExpansion
    );
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::BoundedAncestor]
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &bounded_ancestor_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_marks_non_traversal_detail_as_not_requiring_relationship_proof() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.no-traversal")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.local_detail(
                "user",
                manager_schema(),
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
        .expect("local non-traversal detail read should execute");

    assert!(!result.rows().is_empty());
    assert_relationship_proof_not_required(&result);
}

#[test]
fn compose_read_reports_typed_invalid_root_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.invalid-root")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.local_collection(
                "",
                manager_schema(),
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
        .expect_err("invalid roots should deny before execution");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_eq!(denial.kind(), &WorthQueryReadDenialKind::InvalidRoot);
            assert!(!denial.message().is_empty());
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_local_detail_when_query_classifies_as_broad_search() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.local-broad-mismatch")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.local_detail(
                "user",
                searchable_manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .where_equal(
                            EqualityPredicate::new(
                                "profile",
                                "display_name",
                                WorthQueryPredicateOperand::string("Ada".to_string()),
                            )
                            .expect("equality predicate should build"),
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
        .expect_err("local detail should deny if the query classifies as broad search");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_scope_shape_denial(
                &denial,
                WorthQueryReadScopeClass::LocalNeighborhood,
                WorthQueryReadScopeClass::ExplicitBroadSearch,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}
