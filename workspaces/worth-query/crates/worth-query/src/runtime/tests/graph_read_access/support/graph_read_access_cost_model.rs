use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, OrderingSelector,
    PresencePredicate, RelationName, TraversalSelector, WorthQueryPredicateOperand,
};
use crate::runtime::{
    QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView,
    WorthQueryGraphReadAccessCostEstimate, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadCostAttributionRow, WorthQueryGraphReadMemoryByteEstimate,
    WorthQueryWorkspace,
};

use crate::runtime::tests::graph_read_access::support::public_bridge_runtime::PublicBridgeRuntimeHarness;

pub fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> WorthQueryWorkspace {
    PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace(name)
        .expect("runtime should open workspace")
}

pub fn simple_traversal_family(
    workspace: &mut WorthQueryWorkspace,
    name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(name, |read| {
            read.anchored_collection(
                "user",
                relation_schema(),
                |query| {
                    query
                        .traverse(traversal("manager", 2))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("simple traversal family should admit")
}

pub fn projection_only_family(
    workspace: &mut WorthQueryWorkspace,
    name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(name, |read| {
            read.local_collection(
                "user",
                relation_schema(),
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("projection-only family should admit")
}

pub fn reordered_simple_traversal_family(
    workspace: &mut WorthQueryWorkspace,
    name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(name, |read| {
            read.anchored_collection(
                "user",
                relation_schema(),
                |query| {
                    query
                        .project(field("identity", "id"))
                        .traverse(traversal("manager", 2))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("reordered simple traversal family should admit")
}

pub fn dense_traversal_family(
    workspace: &mut WorthQueryWorkspace,
    name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(name, |read| {
            read.explicit_broad_search_collection(
                "user",
                relation_schema(),
                |query| {
                    query
                        .traverse(traversal("manager", 8))
                        .where_equal(equality("status", "value", "active"))
                        .project(field("identity", "id"))
                        .order_by(
                            OrderingSelector::ascending("profile", "display_name")
                                .expect("ordering should build"),
                        )
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("dense traversal family should admit")
}

pub fn intermediate_pressure_family(
    workspace: &mut WorthQueryWorkspace,
    name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(name, |read| {
            read.explicit_broad_search_collection(
                "user",
                relation_schema(),
                |query| {
                    query
                        .traverse(traversal("manager", 1))
                        .where_present(presence("profile", "display_name"))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("intermediate pressure family should admit")
}

pub fn frontier_search_family(
    workspace: &mut WorthQueryWorkspace,
    name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(name, |read| {
            read.explicit_broad_search_frontier_collection(
                "user",
                two_relation_schema(),
                [relation("manager"), relation("mentor")],
                2,
                |query| {
                    query
                        .project(field("identity", "id"))
                        .where_equal(equality("status", "value", "active"))
                        .order_by(
                            OrderingSelector::ascending("profile", "display_name")
                                .expect("ordering should build"),
                        )
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("frontier search family should admit")
}

pub fn bucket_sum(
    estimate: &WorthQueryGraphReadAccessCostEstimate,
    read_bucket: impl Fn(&WorthQueryGraphReadMemoryByteEstimate) -> usize,
) -> usize {
    estimate
        .attribution_rows()
        .iter()
        .map(|row| read_bucket(row.supported().memory()))
        .sum()
}

pub fn assert_exact_bucket_contribution(
    estimate: &WorthQueryGraphReadAccessCostEstimate,
    kind: WorthQueryGraphReadAccessRequirementKind,
    read_bucket: impl Fn(&WorthQueryGraphReadMemoryByteEstimate) -> usize,
    expected_bytes: usize,
) {
    let row = attribution_row(estimate.attribution_rows(), kind);
    assert_eq!(read_bucket(row.supported().memory()), expected_bytes);
}

fn attribution_row(
    rows: &[WorthQueryGraphReadCostAttributionRow],
    kind: WorthQueryGraphReadAccessRequirementKind,
) -> &WorthQueryGraphReadCostAttributionRow {
    rows.iter()
        .find(|row| row.requirement_kind() == &kind)
        .expect("expected attribution row should exist")
}

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("field selector should build")
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered)
        .expect("result-shape field should build")
}

fn traversal(name: &str, depth: u8) -> TraversalSelector {
    TraversalSelector::bounded(name, depth).expect("traversal selector should build")
}

fn relation(name: &str) -> RelationName {
    RelationName::new(name).expect("relation name should build")
}

fn equality(aspect: &str, field: &str, value: &str) -> EqualityPredicate {
    EqualityPredicate::new(
        aspect,
        field,
        WorthQueryPredicateOperand::string(value.to_string()),
    )
    .expect("equality predicate should build")
}

fn presence(aspect: &str, field: &str) -> PresencePredicate {
    PresencePredicate::is_present(aspect, field).expect("presence predicate should build")
}

fn relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-phase-five-relation",
        [
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("id")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            )
            .presence_predicate_queryable(),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [SchemaRelationView::new(
            crate::facade::foundation::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            8,
        )],
    )
}

fn two_relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-phase-five-two-relation",
        [
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("id")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [
            SchemaRelationView::new(
                crate::facade::foundation::RelationName::new("manager")
                    .expect("schema relation literal must be valid"),
                2,
            ),
            SchemaRelationView::new(
                crate::facade::foundation::RelationName::new("mentor")
                    .expect("schema relation literal must be valid"),
                2,
            ),
        ],
    )
}
