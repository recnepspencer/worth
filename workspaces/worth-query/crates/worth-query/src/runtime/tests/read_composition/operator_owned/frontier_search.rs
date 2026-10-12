use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, OrderingSelector,
    WorthQueryPredicateOperand,
};
use crate::runtime::{
    WorthQueryReadBuiltInOperator, WorthQueryReadBuiltInOperatorDenialReason,
    WorthQueryReadScopeClass, WorthQueryRuntimeError,
};

#[path = "frontier_search/compose_read_executes_operator_owned_frontier_search_detail.rs"]
mod compose_read_executes_operator_owned_frontier_search_detail;

#[test]
fn compose_read_executes_operator_owned_frontier_search_collection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-frontier-search-collection")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_frontier_collection(
                "user",
                searchable_expanded_frontier_manager_schema(),
                [manager_relation_name(), mentor_relation_name()],
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
                        .where_equal(
                            EqualityPredicate::new(
                                "profile",
                                "display_name",
                                WorthQueryPredicateOperand::string("Ada".to_string()),
                            )
                            .expect("equality predicate should build"),
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
        .expect("operator-owned frontier search collection should execute");

    assert!(!result.rows().is_empty());
    assert_collection_receipt(&result, WorthQueryReadScopeClass::ExplicitBroadSearch);
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::FrontierSearch]
    );
}

#[test]
fn compose_read_denies_frontier_search_without_predicate() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-frontier-search-no-predicate")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_frontier_detail(
                "user",
                expanded_frontier_manager_schema(),
                [manager_relation_name(), mentor_relation_name()],
                2,
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
        .expect_err("frontier search should deny when no predicate is declared");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::FrontierSearch,
                WorthQueryReadBuiltInOperatorDenialReason::MissingBroadSearchPredicate,
            );
            assert!(denial.message().contains("requires at least one predicate"));
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_frontier_search_without_relations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-frontier-search-empty")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_frontier_detail(
                "user",
                searchable_expanded_frontier_manager_schema(),
                Vec::new(),
                2,
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
        .expect_err("frontier search should deny empty frontier relations");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::FrontierSearch,
                WorthQueryReadBuiltInOperatorDenialReason::EmptyFrontier,
            );
            assert!(denial.message().contains("frontier search requires"));
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_frontier_search_with_zero_depth() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-frontier-search-zero-depth")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_frontier_detail(
                "user",
                searchable_expanded_frontier_manager_schema(),
                [manager_relation_name(), mentor_relation_name()],
                0,
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
        .expect_err("frontier search should deny zero-depth frontier relations");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::FrontierSearch,
                WorthQueryReadBuiltInOperatorDenialReason::ZeroDepth,
            );
            assert!(denial
                .message()
                .contains("frontier search requires max_depth >= 1"));
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_frontier_search_with_duplicate_relations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-frontier-search-duplicate")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_frontier_collection(
                "user",
                searchable_expanded_frontier_manager_schema(),
                [manager_relation_name(), manager_relation_name()],
                2,
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
        .expect_err("frontier search should deny duplicate frontier relations");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::FrontierSearch,
                WorthQueryReadBuiltInOperatorDenialReason::DuplicateFrontierRelation,
            );
            assert!(denial
                .message()
                .contains("frontier search forbids duplicate frontier relations"));
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_frontier_search_with_degenerate_frontier_shape() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-frontier-search-degenerate")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_frontier_detail(
                "user",
                searchable_expanded_frontier_manager_schema(),
                [manager_relation_name(), mentor_relation_name()],
                1,
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
        .expect_err("frontier search should deny one-hop frontier broad-search shapes");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::FrontierSearch,
                WorthQueryReadBuiltInOperatorDenialReason::DegenerateFrontierShape,
            );
            assert!(denial
                .message()
                .contains("frontier search requires max_depth > 1"));
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}
