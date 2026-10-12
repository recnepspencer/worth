use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, TraversalSelector,
    WorthQueryPredicateOperand,
};
use crate::runtime::{
    WorthQueryReadOperatorFamily, WorthQueryReadScopeClass, WorthQueryRuntimeError,
};

#[test]
fn compose_read_executes_anchored_expansion_detail_with_receipt_classification() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.anchored-expansion")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.anchored_detail(
                "user",
                expanded_manager_schema(),
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
                            TraversalSelector::bounded("manager", 2)
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
        .expect("anchored expansion detail read should execute");

    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::AnchoredExpansion
    );
    assert_eq!(
        result.receipt().breadth().planned_traversal_clause_count(),
        1
    );
    assert_eq!(
        result.receipt().breadth().planned_traversal_depth_limit(),
        2
    );
}

#[test]
fn compose_read_denies_anchored_expansion_without_traversal() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.anchored-denial")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.anchored_detail(
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
        .expect_err("anchored expansion without traversal must deny");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_scope_shape_denial(
                &denial,
                WorthQueryReadScopeClass::AnchoredExpansion,
                WorthQueryReadScopeClass::LocalNeighborhood,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_anchored_expansion_when_shape_is_still_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.anchored-local-mismatch")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.anchored_detail(
                "user",
                manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("manager", 1)
                                .expect("bounded traversal should build"),
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
        .expect_err("anchored expansion should deny if the query still classifies as local");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_scope_shape_denial(
                &denial,
                WorthQueryReadScopeClass::AnchoredExpansion,
                WorthQueryReadScopeClass::LocalNeighborhood,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_executes_explicit_broad_search_detail_with_receipt_classification() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.broad-search")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_detail(
                "user",
                searchable_manager_schema(),
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
        .expect("explicit broad search detail read should execute");

    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::ExplicitBroadSearch
    );
    assert!(result
        .receipt()
        .operator_families()
        .contains(&WorthQueryReadOperatorFamily::Predicate));
}

#[test]
fn compose_read_denies_explicit_broad_search_without_traversal_or_predicate() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.broad-denial")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_detail(
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
        .expect_err("explicit broad search without traversal or predicates must deny");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_scope_shape_denial(
                &denial,
                WorthQueryReadScopeClass::ExplicitBroadSearch,
                WorthQueryReadScopeClass::LocalNeighborhood,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}
