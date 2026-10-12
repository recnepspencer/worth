use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, OrderingSelector, TraversalSelector,
};
use crate::runtime::{WorthQueryReadBuiltInOperator, WorthQueryReadScopeClass};

#[test]
fn compose_read_frontier_detail_matches_open_coded_anchored_detail_query_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.frontier-parity-detail")
        .expect("read-backed runtime should open a workspace");

    let operator_owned = workspace
        .compose_read(request_execution, |read| {
            read.anchored_frontier_detail(
                "user",
                expanded_frontier_manager_schema(),
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
        .expect("operator-owned frontier detail should execute");

    let open_coded = workspace
        .compose_read(request_execution, |read| {
            read.anchored_detail(
                "user",
                expanded_frontier_manager_schema(),
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
        .expect("open-coded anchored detail should execute");

    assert_eq!(operator_owned.rows(), open_coded.rows());
    assert_eq!(
        operator_owned.receipt().scope_class(),
        &WorthQueryReadScopeClass::AnchoredExpansion
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
        [WorthQueryReadBuiltInOperator::AnchoredFrontier]
    );
    assert!(open_coded.receipt().built_in_operator_coverage().is_empty());
}

#[test]
fn compose_read_frontier_collection_matches_open_coded_anchored_collection_query_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.frontier-parity-collection")
        .expect("read-backed runtime should open a workspace");

    let operator_owned = workspace
        .compose_read(request_execution, |read| {
            read.anchored_frontier_collection(
                "user",
                expanded_frontier_manager_schema(),
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
        .expect("operator-owned frontier collection should execute");

    let open_coded = workspace
        .compose_read(request_execution, |read| {
            read.anchored_collection(
                "user",
                expanded_frontier_manager_schema(),
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
        .expect("open-coded anchored collection should execute");

    assert_eq!(operator_owned.rows(), open_coded.rows());
    assert_collection_receipt(&operator_owned, WorthQueryReadScopeClass::AnchoredExpansion);
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
        [WorthQueryReadBuiltInOperator::AnchoredFrontier]
    );
    assert!(open_coded.receipt().built_in_operator_coverage().is_empty());
}
