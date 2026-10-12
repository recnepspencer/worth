use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, OrderingSelector,
    PresencePredicate, RelationName, TraversalSelector, WorthQueryPredicateOperand,
};
use crate::runtime::{
    QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryReadFamily,
    WorthQueryRuntimeSupportProfile, WorthQueryWorkspace,
};

use crate::runtime::tests::graph_read_access::support::graph_index_inventory::runtime_profiles::{
    profile_requiring_store_backed_graph_index, workspace_with_graph_support,
};

pub fn persistent_requirement_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
    profile: WorthQueryRuntimeSupportProfile,
) -> WorthQueryWorkspace {
    workspace_with_graph_support(request_execution, workspace_name, profile)
}

pub fn persistent_predicate_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.explicit_broad_search_collection(
                "user",
                persistent_requirement_schema(),
                |query| {
                    query
                        .traverse(traversal("manager", 1))
                        .where_present(presence("profile", "display_name"))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("persistent predicate family should admit")
}

pub fn reordered_persistent_predicate_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.explicit_broad_search_collection(
                "user",
                persistent_requirement_schema(),
                |query| {
                    query
                        .project(field("identity", "id"))
                        .where_present(presence("profile", "display_name"))
                        .traverse(traversal("manager", 1))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("reordered persistent predicate family should admit")
}

pub fn near_miss_predicate_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.explicit_broad_search_collection(
                "user",
                persistent_requirement_schema(),
                |query| {
                    query
                        .traverse(traversal("manager", 1))
                        .where_equal(equality("status", "value", "active"))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("near-miss predicate family should admit")
}

pub fn broad_persistent_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.explicit_broad_search_collection(
                "user",
                persistent_requirement_schema(),
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
        .expect("broad persistent family should admit")
}

pub fn streaming_frontier_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.explicit_broad_search_frontier_collection(
                "user",
                streaming_frontier_schema(),
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
        .expect("streaming frontier family should admit")
}

pub fn persistent_requirement_digest_for_named_family(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
    family_name: &str,
) -> String {
    let mut workspace = persistent_requirement_workspace(
        request_execution,
        workspace_name,
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = persistent_predicate_family(&mut workspace, family_name);
    persistent_requirement_digest(&mut workspace, &family)
}

pub fn persistent_requirement_digest_for_reordered_family(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
    family_name: &str,
) -> String {
    let mut workspace = persistent_requirement_workspace(
        request_execution,
        workspace_name,
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = reordered_persistent_predicate_family(&mut workspace, family_name);
    persistent_requirement_digest(&mut workspace, &family)
}

pub fn persistent_requirement_digest_for_equality_family(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace_name: &str,
    family_name: &str,
) -> String {
    let mut workspace = persistent_requirement_workspace(
        request_execution,
        workspace_name,
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = near_miss_predicate_family(&mut workspace, family_name);
    persistent_requirement_digest(&mut workspace, &family)
}

fn persistent_requirement_digest(
    workspace: &mut WorthQueryWorkspace,
    family: &WorthQueryReadFamily,
) -> String {
    workspace
        .read_family_intent(family)
        .review()
        .expect("review should derive")
        .graph_read_access_admission()
        .expect("admission should derive")
        .persistent_index_requirement()
        .expect("persistent declaration should exist")
        .digest()
        .to_string()
}

fn persistent_requirement_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-persistent-requirement-schema",
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

fn streaming_frontier_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-persistent-streaming-frontier-schema",
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

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("field selector should build")
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered)
        .expect("result shape field should build")
}

fn traversal(name: &str, depth: u8) -> TraversalSelector {
    TraversalSelector::bounded(name, depth).expect("traversal selector should build")
}

fn presence(aspect: &str, field: &str) -> PresencePredicate {
    PresencePredicate::is_present(aspect, field).expect("presence predicate should build")
}

fn equality(aspect: &str, field: &str, value: &str) -> EqualityPredicate {
    EqualityPredicate::new(
        aspect,
        field,
        WorthQueryPredicateOperand::string(value.to_string()),
    )
    .expect("equality predicate should build")
}

fn relation(name: &str) -> RelationName {
    RelationName::new(name).expect("relation name should build")
}
