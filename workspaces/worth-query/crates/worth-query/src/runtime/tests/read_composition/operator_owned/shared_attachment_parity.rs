use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, OrderingSelector, TraversalSelector,
};
use crate::runtime::{WorthQueryReadBuiltInOperator, WorthQueryReadScopeClass};

#[test]
fn compose_read_shared_attachment_detail_matches_open_coded_local_detail_query_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.shared-attachment-parity-detail")
        .expect("read-backed runtime should open a workspace");

    let operator_owned = workspace
        .compose_read(request_execution, |read| {
            read.local_shared_attachment_detail(
                "user",
                frontier_manager_schema(),
                [manager_relation_name(), mentor_relation_name()],
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
        .expect("operator-owned shared attachment detail should execute");

    let open_coded = workspace
        .compose_read(request_execution, |read| {
            read.local_detail(
                "user",
                frontier_manager_schema(),
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
                                .expect("manager traversal should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("mentor", 1)
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
        .expect("open-coded local detail should execute");

    assert_eq!(operator_owned.rows(), open_coded.rows());
    assert_eq!(
        operator_owned.receipt().scope_class(),
        &WorthQueryReadScopeClass::LocalNeighborhood
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
        [WorthQueryReadBuiltInOperator::SharedAttachment]
    );
    assert!(open_coded.receipt().built_in_operator_coverage().is_empty());
}

#[test]
fn compose_read_shared_attachment_collection_matches_open_coded_local_collection_query_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.shared-attachment-parity-collection")
        .expect("read-backed runtime should open a workspace");

    let operator_owned = workspace
        .compose_read(request_execution, |read| {
            read.local_shared_attachment_collection(
                "user",
                frontier_manager_schema(),
                [manager_relation_name(), mentor_relation_name()],
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
        .expect("operator-owned shared attachment collection should execute");

    let open_coded = workspace
        .compose_read(request_execution, |read| {
            read.local_collection(
                "user",
                frontier_manager_schema(),
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
                                .expect("manager traversal should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("mentor", 1)
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
        .expect("open-coded local collection should execute");

    assert_eq!(operator_owned.rows(), open_coded.rows());
    assert_collection_receipt(&operator_owned, WorthQueryReadScopeClass::LocalNeighborhood);
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
        [WorthQueryReadBuiltInOperator::SharedAttachment]
    );
    assert!(open_coded.receipt().built_in_operator_coverage().is_empty());
}
