use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, OrderingSelector,
    TraversalSelector, WorthQueryPredicateOperand,
};
use crate::runtime::{WorthQueryReadBuiltInOperator, WorthQueryReadScopeClass};

#[test]
fn compose_read_frontier_search_detail_matches_open_coded_broad_search_detail_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.frontier-search-parity-detail")
        .expect("read-backed runtime should open a workspace");

    let operator_owned = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_frontier_detail(
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
        .expect("operator-owned frontier search detail should execute");

    let open_coded = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_detail(
                "user",
                searchable_expanded_frontier_manager_schema(),
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
                                .expect("manager traversal should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("mentor", 2)
                                .expect("mentor traversal should build"),
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
        .expect("open-coded broad search detail should execute");

    assert_eq!(operator_owned.rows(), open_coded.rows());
    assert_eq!(
        operator_owned.receipt().scope_class(),
        &WorthQueryReadScopeClass::ExplicitBroadSearch
    );
    assert_eq!(
        operator_owned.receipt().query_digest(),
        open_coded.receipt().query_digest()
    );
    assert_eq!(
        operator_owned.receipt().basis_digest(),
        open_coded.receipt().basis_digest()
    );
    assert_eq!(
        operator_owned.receipt().result_digest(),
        open_coded.receipt().result_digest()
    );
    assert_eq!(
        operator_owned.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::FrontierSearch]
    );
    assert!(open_coded.receipt().built_in_operator_coverage().is_empty());
}

#[test]
fn compose_read_frontier_search_collection_matches_open_coded_broad_search_collection_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.frontier-search-parity-collection")
        .expect("read-backed runtime should open a workspace");

    let operator_owned = workspace
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

    let open_coded = workspace
        .compose_read(request_execution, |read| {
            read.explicit_broad_search_collection(
                "user",
                searchable_expanded_frontier_manager_schema(),
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
                                .expect("manager traversal should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("mentor", 2)
                                .expect("mentor traversal should build"),
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
        .expect("open-coded broad search collection should execute");

    assert_eq!(operator_owned.rows(), open_coded.rows());
    assert_collection_receipt(
        &operator_owned,
        WorthQueryReadScopeClass::ExplicitBroadSearch,
    );
    assert_eq!(
        operator_owned.receipt().query_digest(),
        open_coded.receipt().query_digest()
    );
    assert_eq!(
        operator_owned.receipt().basis_digest(),
        open_coded.receipt().basis_digest()
    );
    assert_eq!(
        operator_owned.receipt().result_digest(),
        open_coded.receipt().result_digest()
    );
    assert_eq!(
        operator_owned.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::FrontierSearch]
    );
    assert!(open_coded.receipt().built_in_operator_coverage().is_empty());
}
