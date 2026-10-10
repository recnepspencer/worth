use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, RelationName,
    WorthQueryPredicateOperand,
};
use crate::runtime::{
    explain_graph_read_access_requirements_for_family, QuerySchemaView, ScalarAspectType,
    SchemaFieldView, SchemaRelationView, WorthQueryGraphReadAccessRequirementKind,
};

use crate::runtime::tests::graph_read_access::support;

use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn operator_matrix_derives_the_expected_structural_requirement_kinds() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for (name, required_kinds) in operator_cases() {
        let mut workspace = workspace(
            request_execution,
            &format!("graph-read-access.phase-four.{name}"),
        );
        let family = operator_family(&mut workspace, name);
        let requirements = explain_graph_read_access_requirements_for_family(&family)
            .expect("operator requirements should derive");

        for required_kind in required_kinds {
            assert!(
                requirements.requires_kind(required_kind.clone()),
                "{name} should require {}",
                required_kind.as_str()
            );
        }
    }
}

fn operator_cases() -> Vec<(&'static str, Vec<WorthQueryGraphReadAccessRequirementKind>)> {
    use WorthQueryGraphReadAccessRequirementKind as Kind;
    vec![
        (
            "direct-edge",
            vec![Kind::DirectionalAdjacency, Kind::ResultBuffer],
        ),
        (
            "successor-walk",
            vec![
                Kind::DirectionalAdjacency,
                Kind::TraversalWorkset,
                Kind::VisitedSet,
            ],
        ),
        (
            "bounded-ancestor",
            vec![
                Kind::ReverseAdjacency,
                Kind::TraversalWorkset,
                Kind::VisitedSet,
            ],
        ),
        (
            "bounded-descendant",
            vec![
                Kind::DirectionalAdjacency,
                Kind::TraversalWorkset,
                Kind::VisitedSet,
            ],
        ),
        (
            "anchored-frontier",
            vec![
                Kind::DirectionalAdjacency,
                Kind::TraversalWorkset,
                Kind::DedupSet,
            ],
        ),
        (
            "shared-endpoint",
            vec![
                Kind::ReverseAdjacency,
                Kind::TraversalWorkset,
                Kind::DedupSet,
            ],
        ),
        (
            "shared-attachment",
            vec![
                Kind::ReverseAdjacency,
                Kind::TraversalWorkset,
                Kind::DedupSet,
            ],
        ),
        (
            "frontier-search",
            vec![
                Kind::DirectionalAdjacency,
                Kind::ReverseAdjacency,
                Kind::DedupSet,
            ],
        ),
    ]
}

fn operator_family(
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(name, |read| match name {
            "direct-edge" => read.local_direct_edge_collection(
                "user",
                two_relation_schema(),
                relation("manager"),
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            "successor-walk" => read.local_successor_walk_collection(
                "user",
                two_relation_schema(),
                relation("manager"),
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            "bounded-ancestor" => read.anchored_bounded_ancestor_collection(
                "user",
                two_relation_schema(),
                relation("manager"),
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            "bounded-descendant" => read.anchored_bounded_descendant_collection(
                "user",
                two_relation_schema(),
                relation("manager"),
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            "anchored-frontier" => read.anchored_frontier_collection(
                "user",
                two_relation_schema(),
                [relation("manager"), relation("mentor")],
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            "shared-endpoint" => read.local_shared_endpoint_collection(
                "user",
                two_relation_schema(),
                [relation("manager"), relation("mentor")],
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            "shared-attachment" => read.local_shared_attachment_collection(
                "user",
                two_relation_schema(),
                [relation("manager"), relation("mentor")],
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            "frontier-search" => read.explicit_broad_search_frontier_collection(
                "user",
                two_relation_schema(),
                [relation("manager"), relation("mentor")],
                2,
                |query| {
                    query
                        .project(field("identity", "id"))
                        .where_equal(equality("status", "value", "active"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            ),
            _ => unreachable!("operator case should be declared above"),
        })
        .expect("operator family should admit")
}

fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> crate::runtime::WorthQueryWorkspace {
    PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace(name)
        .expect("runtime should open workspace")
}

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("field selector should build")
}

fn relation(name: &str) -> RelationName {
    RelationName::new(name).expect("relation name should build")
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered)
        .expect("result-shape field should build")
}

fn equality(aspect: &str, field: &str, value: &str) -> EqualityPredicate {
    EqualityPredicate::new(
        aspect,
        field,
        WorthQueryPredicateOperand::string(value.to_string()),
    )
    .expect("equality predicate should build")
}

fn two_relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-phase-four-two-relation",
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
