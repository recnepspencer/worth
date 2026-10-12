use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, CollectionQueryBuilder, EqualityPredicate,
    NativeComparisonPredicate, OrderingSelector, PresencePredicate, RelationName,
    SetMembershipPredicate, StringContainsPredicate, WorthQueryPredicateOperand,
};
use crate::runtime::{
    QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView,
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryGraphReadAccessShape,
    WorthQueryGraphReadFanoutPosture, WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadPredicateFamily, WorthQueryGraphReadResultPressure,
    WorthQueryGraphReadRootPosture, WorthQueryGraphReadTraversalOperator, WorthQueryReadScopeClass,
};

use crate::runtime::tests::graph_read_access::support;

use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn graph_read_access_shape_changes_for_relation_direction_and_depth() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-one.direction-depth")
        .expect("runtime should open workspace");
    let ancestor = workspace
        .define_read_family("ancestor", |read| {
            read.anchored_bounded_ancestor_collection(
                "user",
                relation_schema(),
                relation("manager"),
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("ancestor family should be admitted");
    let descendant = workspace
        .define_read_family("descendant", |read| {
            read.anchored_bounded_descendant_collection(
                "user",
                relation_schema(),
                relation("manager"),
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("descendant family should be admitted");
    let deeper_ancestor = workspace
        .define_read_family("deeper-ancestor", |read| {
            read.anchored_bounded_ancestor_collection(
                "user",
                relation_schema(),
                relation("manager"),
                3,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("deeper ancestor family should be admitted");

    let ancestor_shape = access_shape(&workspace, &ancestor);
    let descendant_shape = access_shape(&workspace, &descendant);
    let deeper_shape = access_shape(&workspace, &deeper_ancestor);

    assert_ne!(ancestor_shape.digest(), descendant_shape.digest());
    assert_ne!(ancestor_shape.digest(), deeper_shape.digest());
    assert_shape_dimensions(
        &ancestor_shape,
        ExpectedGraphReadShape {
            root: WorthQueryGraphReadRootPosture::Anchored,
            scope: WorthQueryReadScopeClass::AnchoredExpansion,
            operators: &[WorthQueryGraphReadTraversalOperator::BoundedAncestor],
            directions: &[WorthQueryAdmittedGraphReadRelationDirection::Ancestor],
            max_depth: 2,
            fanout: WorthQueryGraphReadFanoutPosture::SingleRelation,
            predicate: WorthQueryGraphReadPredicateFamily::None,
            ordering: WorthQueryGraphReadOrderingPosture::ProviderOrdered,
            result: WorthQueryGraphReadResultPressure::CollectionWide,
        },
    );
    assert_eq!(
        descendant_shape.relation_directions(),
        [WorthQueryAdmittedGraphReadRelationDirection::Descendant]
    );
    assert_eq!(deeper_shape.max_depth(), 3);
}

#[test]
fn graph_read_access_shape_classifies_predicate_families_and_mixed_predicates() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let equality = predicate_shape(request_execution, "equality", |query| {
        query
            .where_equal(
                EqualityPredicate::new(
                    "status",
                    "value",
                    WorthQueryPredicateOperand::string("active".to_string()),
                )
                .expect("equality predicate should build"),
            )
            .project(field("identity", "id"))
    });
    let range = predicate_shape(request_execution, "range", |query| {
        query
            .where_greater_than(
                NativeComparisonPredicate::greater_than("profile", "age", 21)
                    .expect("range predicate should build"),
            )
            .project(field("identity", "id"))
    });
    let text = predicate_shape(request_execution, "text", |query| {
        query
            .where_contains(
                StringContainsPredicate::new("profile", "display_name", "Ada")
                    .expect("text predicate should build"),
            )
            .project(field("identity", "id"))
    });
    let membership = predicate_shape(request_execution, "membership", |query| {
        query
            .where_in(
                SetMembershipPredicate::new(
                    "status",
                    "value",
                    [WorthQueryPredicateOperand::string("active".to_string())],
                )
                .expect("membership predicate should build"),
            )
            .project(field("identity", "id"))
    });
    let presence = predicate_shape(request_execution, "presence", |query| {
        query
            .where_present(
                PresencePredicate::is_present("profile", "display_name")
                    .expect("presence predicate should build"),
            )
            .project(field("identity", "id"))
    });
    let mixed = predicate_shape(request_execution, "mixed", |query| {
        query
            .where_contains(
                StringContainsPredicate::new("profile", "display_name", "Ada")
                    .expect("text predicate should build"),
            )
            .where_greater_than(
                NativeComparisonPredicate::greater_than("profile", "age", 21)
                    .expect("range predicate should build"),
            )
            .project(field("identity", "id"))
    });

    assert_eq!(
        equality.predicate_family(),
        &WorthQueryGraphReadPredicateFamily::Equality
    );
    assert_eq!(
        range.predicate_family(),
        &WorthQueryGraphReadPredicateFamily::Range
    );
    assert_eq!(
        text.predicate_family(),
        &WorthQueryGraphReadPredicateFamily::Text
    );
    assert_eq!(
        membership.predicate_family(),
        &WorthQueryGraphReadPredicateFamily::Membership
    );
    assert_eq!(
        presence.predicate_family(),
        &WorthQueryGraphReadPredicateFamily::Presence
    );
    assert_eq!(
        mixed.predicate_family(),
        &WorthQueryGraphReadPredicateFamily::Mixed
    );
}

#[test]
fn graph_read_access_shape_distinguishes_canonical_and_explicit_ordering() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-one.ordering")
        .expect("runtime should open workspace");
    let canonical_ordered = workspace
        .define_read_family("canonical-ordered", |read| {
            read.local_collection(
                "user",
                predicate_schema(),
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("canonically ordered family should be admitted");
    let ordered = workspace
        .define_read_family("ordered", |read| {
            read.local_collection(
                "user",
                predicate_schema(),
                |query| {
                    query.project(field("identity", "id")).order_by(
                        OrderingSelector::ascending("profile", "display_name")
                            .expect("ordering should build"),
                    )
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("ordered family should be admitted");

    let canonical_shape = access_shape(&workspace, &canonical_ordered);
    let ordered_shape = access_shape(&workspace, &ordered);

    assert_ne!(canonical_shape.digest(), ordered_shape.digest());
    assert_eq!(
        canonical_shape.ordering_posture(),
        &WorthQueryGraphReadOrderingPosture::ProviderOrdered
    );
    assert_eq!(
        ordered_shape.ordering_posture(),
        &WorthQueryGraphReadOrderingPosture::ProviderOrdered
    );
    let canonical_orderings = canonical_shape
        .operation_resolution()
        .references()
        .orderings();
    let explicit_orderings = ordered_shape
        .operation_resolution()
        .references()
        .orderings();
    assert_eq!(canonical_orderings.len(), 1);
    assert_eq!(
        canonical_orderings[0].native_aspect_key().as_str(),
        "identity"
    );
    assert_eq!(canonical_orderings[0].native_field_key().as_str(), "id");
    assert_eq!(canonical_orderings[0].direction(), "ascending");
    assert_eq!(explicit_orderings.len(), 1);
    assert_eq!(
        explicit_orderings[0].native_aspect_key().as_str(),
        "profile"
    );
    assert_eq!(
        explicit_orderings[0].native_field_key().as_str(),
        "display_name"
    );
}

struct ExpectedGraphReadShape<'a> {
    root: WorthQueryGraphReadRootPosture,
    scope: WorthQueryReadScopeClass,
    operators: &'a [WorthQueryGraphReadTraversalOperator],
    directions: &'a [WorthQueryAdmittedGraphReadRelationDirection],
    max_depth: usize,
    fanout: WorthQueryGraphReadFanoutPosture,
    predicate: WorthQueryGraphReadPredicateFamily,
    ordering: WorthQueryGraphReadOrderingPosture,
    result: WorthQueryGraphReadResultPressure,
}

fn assert_shape_dimensions(
    shape: &WorthQueryGraphReadAccessShape,
    expected: ExpectedGraphReadShape<'_>,
) {
    assert_eq!(shape.root_posture(), &expected.root);
    assert_eq!(shape.scope_class(), &expected.scope);
    assert_eq!(shape.traversal_operators(), expected.operators);
    assert_eq!(shape.relation_directions(), expected.directions);
    assert_eq!(shape.max_depth(), expected.max_depth);
    assert_eq!(shape.fanout_posture(), &expected.fanout);
    assert_eq!(shape.predicate_family(), &expected.predicate);
    assert_eq!(shape.ordering_posture(), &expected.ordering);
    assert_eq!(shape.result_pressure(), &expected.result);
}

fn access_shape<'a>(
    workspace: &'a crate::runtime::WorthQueryWorkspace,
    family: &'a crate::runtime::WorthQueryReadFamily,
) -> WorthQueryGraphReadAccessShape {
    let explanation = workspace
        .explain_graph_read_access_shape(family)
        .expect("access shape should explain");
    explanation.access_shape().clone()
}

fn predicate_shape(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
    declare_query: impl FnOnce(CollectionQueryBuilder) -> CollectionQueryBuilder,
) -> WorthQueryGraphReadAccessShape {
    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace(format!("graph-read-access.phase-one.predicate.{label}"))
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
    let explanation = workspace
        .explain_graph_read_access_shape(&family)
        .expect("predicate access shape should explain");
    explanation.access_shape().clone()
}

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("field selector should build")
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered)
        .expect("result-shape field should build")
}

fn relation(name: &str) -> RelationName {
    RelationName::new(name).expect("relation name should build")
}

fn relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-direction",
        [SchemaFieldView::new(
            crate::facade::foundation::AspectName::new("identity")
                .expect("schema aspect literal must be valid"),
            crate::facade::foundation::FieldName::new("id")
                .expect("schema field literal must be valid"),
            ScalarAspectType::String,
        )],
        [SchemaRelationView::new(
            crate::facade::foundation::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            3,
        )],
    )
}

fn predicate_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-predicate",
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
