use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, CollectionQueryBuilder, EqualityPredicate,
    NativeComparisonPredicate, PresencePredicate, SetMembershipPredicate, StringContainsPredicate,
    WorthQueryPredicateOperand,
};
use crate::runtime::{
    QuerySchemaView, ScalarAspectType, SchemaFieldView, WorthQueryBooleanPredicateTopology,
    WorthQueryBooleanSelectivityAdmissionPosture, WorthQueryBooleanSelectivityBranchKind,
    WorthQueryBooleanSelectivityShape, WorthQueryPredicateAnchorPosture,
    WorthQueryPredicateOperandOperator, WorthQueryPredicateSelectivityClass,
    WorthQueryTraversalPredicateOrderingPosture,
};

use crate::runtime::tests::graph_read_access::support;

use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn boolean_selectivity_shape_normalizes_flat_conjunctive_predicates_without_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let shape = selectivity_shape(request_execution, "mixed", |query| {
        query
            .where_equal(equality("status", "value", "active"))
            .where_greater_than(
                NativeComparisonPredicate::greater_than("profile", "age", 21)
                    .expect("range predicate should build"),
            )
            .where_contains(
                StringContainsPredicate::new("profile", "display_name", "Ada")
                    .expect("text predicate should build"),
            )
            .project(field("identity", "id"))
    });

    assert_eq!(
        shape.boolean_topology(),
        &WorthQueryBooleanPredicateTopology::ConjunctiveFlat
    );
    assert_eq!(
        shape.anchor_posture(),
        &WorthQueryPredicateAnchorPosture::MixedAnchorAndBroad
    );
    assert_eq!(
        shape.traversal_predicate_ordering_posture(),
        &WorthQueryTraversalPredicateOrderingPosture::Mixed
    );
    assert_eq!(shape.counters().predicate_rows_normalized(), 3);
    assert_eq!(shape.counters().exact_predicate_count(), 1);
    assert_eq!(shape.counters().broad_predicate_count(), 1);
    assert_eq!(shape.counters().risky_predicate_count(), 0);
    assert_eq!(shape.counters().pre_traversal_eligible_count(), 2);
    assert_eq!(shape.counters().expression_nodes_visited(), 1);
    assert_eq!(shape.counters().branches_produced(), 1);
    assert_eq!(shape.counters().executor_observations_consumed(), 0);
    assert_eq!(
        shape.admission_posture(),
        &WorthQueryBooleanSelectivityAdmissionPosture::InlineEligible
    );
    assert_eq!(shape.branches().len(), 1);
    assert_eq!(
        shape.branches()[0].branch_kind(),
        &WorthQueryBooleanSelectivityBranchKind::ConjunctiveRoot
    );
    assert_eq!(shape.branches()[0].expression_path(), "root");
    assert_eq!(shape.branches()[0].predicate_rows().len(), 3);
    assert!(has_selectivity_class(
        &shape,
        WorthQueryPredicateSelectivityClass::ExactAnchor
    ));
    assert!(has_selectivity_class(
        &shape,
        WorthQueryPredicateSelectivityClass::RangePredicate
    ));
    assert!(has_selectivity_class(
        &shape,
        WorthQueryPredicateSelectivityClass::BroadPredicate
    ));
}

#[test]
fn boolean_selectivity_shape_canonicalizes_predicate_order_and_membership_values() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = selectivity_shape(request_execution, "canonical-a", |query| {
        query
            .where_in(membership(["active", "pending"]))
            .where_equal(equality("identity", "id", "user-1"))
            .project(field("identity", "id"))
    });
    let second = selectivity_shape(request_execution, "canonical-b", |query| {
        query
            .where_equal(equality("identity", "id", "user-1"))
            .where_in(membership(["pending", "active"]))
            .project(field("identity", "id"))
    });

    assert_eq!(
        first.digest(),
        second.digest(),
        "semantically equivalent predicate order should produce one planning shape"
    );
    assert_eq!(
        operand_identities(&first),
        operand_identities(&second),
        "membership operand identity should be canonicalized before planning"
    );
}

#[test]
fn boolean_selectivity_shape_preserves_same_field_distinct_range_operand_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let shape = selectivity_shape(request_execution, "distinct-operands", |query| {
        query
            .where_greater_than(
                NativeComparisonPredicate::greater_than("profile", "age", 18)
                    .expect("lower bound predicate should build"),
            )
            .where_less_than(
                NativeComparisonPredicate::less_than("profile", "age", 65)
                    .expect("upper bound predicate should build"),
            )
            .project(field("identity", "id"))
    });
    assert_eq!(
        structured_operands(&shape),
        vec![
            (
                WorthQueryPredicateOperandOperator::GreaterThan,
                vec![predicate_operand_identity(
                    &WorthQueryPredicateOperand::int64(18),
                )]
            ),
            (
                WorthQueryPredicateOperandOperator::LessThan,
                vec![predicate_operand_identity(
                    &WorthQueryPredicateOperand::int64(65),
                )]
            ),
        ]
    );
    assert_eq!(shape.counters().pre_traversal_eligible_count(), 2);
}

#[test]
fn boolean_selectivity_shape_reports_broad_only_presence_as_post_traversal() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let shape = selectivity_shape(request_execution, "presence", |query| {
        query
            .where_present(
                PresencePredicate::is_present("profile", "display_name")
                    .expect("presence predicate should build"),
            )
            .project(field("identity", "id"))
    });

    assert_eq!(
        shape.anchor_posture(),
        &WorthQueryPredicateAnchorPosture::BroadOnly
    );
    assert_eq!(
        shape.traversal_predicate_ordering_posture(),
        &WorthQueryTraversalPredicateOrderingPosture::PostTraversalFilterRequired
    );
    assert_eq!(shape.counters().predicate_rows_normalized(), 1);
    assert_eq!(shape.counters().broad_predicate_count(), 1);
    assert_eq!(shape.counters().pre_traversal_eligible_count(), 0);
    assert_eq!(
        shape.predicate_rows()[0].selectivity_class(),
        &WorthQueryPredicateSelectivityClass::PostTraversalOnly
    );
    assert_eq!(
        shape.predicate_rows()[0].operator(),
        &WorthQueryPredicateOperandOperator::Presence
    );
    assert_eq!(
        shape.predicate_rows()[0].normalized_operand_values(),
        &["is_present".to_string()]
    );
}

#[test]
fn boolean_selectivity_shape_rejects_unadmitted_predicate_before_normalization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-two.selectivity.schema-rejection")
        .expect("runtime should open workspace");
    let error = workspace
        .define_read_family("schema-rejection", |read| {
            read.explicit_broad_search_collection(
                "user",
                QuerySchemaView::new(
                    "graph-read-access-selectivity-rejection",
                    [SchemaFieldView::new(
                        crate::facade::foundation::AspectName::new("identity")
                            .expect("schema aspect literal must be valid"),
                        crate::facade::foundation::FieldName::new("id")
                            .expect("schema field literal must be valid"),
                        ScalarAspectType::String,
                    )],
                    [],
                ),
                |query| {
                    query
                        .where_equal(equality("status", "value", "active"))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect_err("unadmitted predicate must not define a read family");

    assert!(format!("{error:?}").contains("UnknownField"));
}

fn selectivity_shape(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
    declare_query: impl FnOnce(CollectionQueryBuilder) -> CollectionQueryBuilder,
) -> WorthQueryBooleanSelectivityShape {
    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace(format!("graph-read-access.phase-two.selectivity.{label}"))
        .expect("runtime should open workspace");
    let family = workspace
        .define_read_family(label, |read| {
            read.explicit_broad_search_collection(
                "user",
                predicate_schema(),
                declare_query,
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("predicate family should be admitted");
    workspace
        .explain_boolean_selectivity_shape(&family)
        .expect("boolean selectivity shape should explain")
}

fn has_selectivity_class(
    shape: &WorthQueryBooleanSelectivityShape,
    class: WorthQueryPredicateSelectivityClass,
) -> bool {
    shape
        .predicate_rows()
        .iter()
        .any(|row| row.selectivity_class() == &class)
}

fn operand_identities(shape: &WorthQueryBooleanSelectivityShape) -> Vec<String> {
    shape
        .predicate_rows()
        .iter()
        .map(|row| row.operand_identity().to_string())
        .collect()
}

fn structured_operands(
    shape: &WorthQueryBooleanSelectivityShape,
) -> Vec<(WorthQueryPredicateOperandOperator, Vec<String>)> {
    shape
        .predicate_rows()
        .iter()
        .map(|row| {
            (
                row.operator().clone(),
                row.normalized_operand_values().to_vec(),
            )
        })
        .collect()
}

fn predicate_operand_identity(value: &WorthQueryPredicateOperand) -> String {
    worth_foundational::facade::prepare_aspect_value_identity_basis(value.as_native())
        .as_str()
        .to_owned()
}

fn equality(aspect: &str, field: &str, value: &str) -> EqualityPredicate {
    EqualityPredicate::new(
        aspect,
        field,
        WorthQueryPredicateOperand::string(value.to_string()),
    )
    .expect("equality predicate should build")
}

fn membership<const N: usize>(values: [&str; N]) -> SetMembershipPredicate {
    SetMembershipPredicate::new(
        "status",
        "value",
        values
            .into_iter()
            .map(|value| WorthQueryPredicateOperand::string(value.to_string())),
    )
    .expect("membership predicate should build")
}

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("field selector should build")
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered)
        .expect("result-shape field should build")
}

fn predicate_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-selectivity",
        [
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("id")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            )
            .membership_predicate_queryable(),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("age")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::Int64,
            ),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            )
            .text_predicate_queryable()
            .presence_predicate_queryable(),
        ],
        [],
    )
}
