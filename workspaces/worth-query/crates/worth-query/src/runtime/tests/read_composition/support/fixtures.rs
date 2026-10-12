use super::super::super::support::*;
use crate::authoring::RelationName;
use crate::schema_view::{QuerySchemaView, SchemaRelationView};

pub(crate) fn read_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    let mut runtime = stateful_bridge_runtime_with_collections(request_execution, &["user"]);
    runtime
        .write(
            request_execution,
            insert_command(
                "user",
                [
                    ("identity.id", test_string_aspect_value("user-1")),
                    ("profile.display_name", test_string_aspect_value("Ada")),
                ],
            ),
        )
        .expect("read-composition fixture should seed an authoritative user row");
    runtime
}

pub(crate) fn manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "runtime-read-composition",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [SchemaRelationView::new(
            crate::authoring::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            1,
        )],
    )
}

pub(crate) fn expanded_manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "runtime-read-composition-expanded",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [SchemaRelationView::new(
            crate::authoring::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            2,
        )],
    )
}

pub(crate) fn frontier_manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "runtime-read-composition-frontier",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [
            SchemaRelationView::new(
                crate::authoring::RelationName::new("manager")
                    .expect("schema relation literal must be valid"),
                1,
            ),
            SchemaRelationView::new(
                crate::authoring::RelationName::new("mentor")
                    .expect("schema relation literal must be valid"),
                1,
            ),
        ],
    )
}

pub(crate) fn expanded_frontier_manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "runtime-read-composition-frontier-expanded",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [
            SchemaRelationView::new(
                crate::authoring::RelationName::new("manager")
                    .expect("schema relation literal must be valid"),
                2,
            ),
            SchemaRelationView::new(
                crate::authoring::RelationName::new("mentor")
                    .expect("schema relation literal must be valid"),
                2,
            ),
        ],
    )
}

pub(crate) fn searchable_manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "runtime-read-composition-searchable",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            )
            .text_predicate_queryable()
            .membership_predicate_queryable()
            .presence_predicate_queryable(),
        ],
        [SchemaRelationView::new(
            crate::authoring::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            1,
        )],
    )
}

pub(crate) fn searchable_expanded_frontier_manager_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "runtime-read-composition-searchable-frontier-expanded",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            )
            .text_predicate_queryable()
            .membership_predicate_queryable()
            .presence_predicate_queryable(),
        ],
        [
            SchemaRelationView::new(
                crate::authoring::RelationName::new("manager")
                    .expect("schema relation literal must be valid"),
                2,
            ),
            SchemaRelationView::new(
                crate::authoring::RelationName::new("mentor")
                    .expect("schema relation literal must be valid"),
                2,
            ),
        ],
    )
}

pub(crate) fn manager_relation_name() -> RelationName {
    RelationName::new("manager").expect("test relation name should validate")
}

pub(crate) fn mentor_relation_name() -> RelationName {
    RelationName::new("mentor").expect("test relation name should validate")
}
