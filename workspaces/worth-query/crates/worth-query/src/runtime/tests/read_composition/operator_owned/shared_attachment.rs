use super::super::support::*;
use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, OrderingSelector,
    WorthQueryPredicateOperand,
};
use crate::runtime::{
    WorthQueryReadBuiltInOperator, WorthQueryReadBuiltInOperatorDenialReason,
    WorthQueryReadScopeClass, WorthQueryRuntimeError,
};

#[test]
fn compose_read_executes_operator_owned_shared_attachment_detail() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-shared-attachment-detail")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
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

    assert!(!result.rows().is_empty());
    assert_eq!(
        result.receipt().scope_class(),
        &WorthQueryReadScopeClass::LocalNeighborhood
    );
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::SharedAttachment]
    );
    assert_eq!(
        result.receipt().breadth().planned_traversal_clause_count(),
        2
    );
    assert_eq!(
        result.receipt().breadth().planned_traversal_depth_limit(),
        1
    );
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        3,
        &direct_edge_synthetic_runtime_surfaces(),
    );
}

#[test]
fn compose_read_executes_operator_owned_shared_attachment_collection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-shared-attachment-collection")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
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

    assert!(!result.rows().is_empty());
    assert_collection_receipt(&result, WorthQueryReadScopeClass::LocalNeighborhood);
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::SharedAttachment]
    );
}

#[test]
fn compose_read_keeps_identity_anchored_shared_attachment_collection_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-shared-attachment-identity-anchor")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
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
                        .where_equal(
                            EqualityPredicate::new(
                                "identity",
                                "id",
                                WorthQueryPredicateOperand::string("user-ada".to_string()),
                            )
                            .expect("identity anchor predicate should build"),
                        )
                        .order_by(
                            OrderingSelector::ascending("identity", "id")
                                .expect("ordering should build"),
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
        .expect("identity-anchored shared attachment collection should stay local");

    assert_collection_receipt(&result, WorthQueryReadScopeClass::LocalNeighborhood);
    assert_eq!(
        result.receipt().built_in_operator_coverage(),
        [WorthQueryReadBuiltInOperator::SharedAttachment]
    );
}

#[test]
fn compose_read_denies_shared_attachment_without_enough_relations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-shared-attachment-too-small")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.local_shared_attachment_detail(
                "user",
                frontier_manager_schema(),
                [manager_relation_name()],
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
        .expect_err("shared attachment should deny single-relation shapes");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::SharedAttachment,
                WorthQueryReadBuiltInOperatorDenialReason::TooFewSharedRelations,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}

#[test]
fn compose_read_denies_shared_attachment_with_duplicate_relations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.operator-shared-attachment-duplicate")
        .expect("read-backed runtime should open a workspace");

    let error = workspace
        .compose_read(request_execution, |read| {
            read.local_shared_attachment_collection(
                "user",
                frontier_manager_schema(),
                [manager_relation_name(), manager_relation_name()],
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
        .expect_err("shared attachment should deny duplicate relation sets");

    match error {
        WorthQueryRuntimeError::ReadCompositionDenied(denial) => {
            assert_built_in_operator_denial(
                &denial,
                WorthQueryReadBuiltInOperator::SharedAttachment,
                WorthQueryReadBuiltInOperatorDenialReason::DuplicateSharedRelation,
            );
        }
        other => panic!("expected typed read-composition denial, got {other:?}"),
    }
}
