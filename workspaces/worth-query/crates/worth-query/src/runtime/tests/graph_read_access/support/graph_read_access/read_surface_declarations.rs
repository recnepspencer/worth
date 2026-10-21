use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, TraversalSelector,
    WorthQueryGraphReadDomainOperationDeclaration,
};
use crate::runtime::{
    QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView, WorthQueryReadFamily,
    WorthQueryReadGraph, WorthQueryReadResult, WorthQueryRuntimeError, WorthQueryWorkspace,
};

pub fn graph_access_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, graph_access_collection_read_graph)
        .expect("graph-access family should admit")
}

pub fn execute_graph_access_compose_read(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
    workspace.compose_read(request_execution, graph_access_collection_read_graph)
}

pub fn unregistered_domain_operation_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, unregistered_domain_operation_read_graph)
        .expect("domain operation family should define before access admission")
}

pub fn execute_unregistered_domain_operation_compose_read(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
) -> Result<WorthQueryReadResult, WorthQueryRuntimeError> {
    workspace.compose_read(request_execution, unregistered_domain_operation_read_graph)
}

fn graph_access_collection_read_graph(
    read: crate::runtime::WorthQueryReadBuilder,
) -> Result<WorthQueryReadGraph, crate::runtime::WorthQueryReadDenial> {
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
}

fn unregistered_domain_operation_read_graph(
    read: crate::runtime::WorthQueryReadBuilder,
) -> Result<WorthQueryReadGraph, crate::runtime::WorthQueryReadDenial> {
    read.anchored_collection(
        "user",
        relation_schema(),
        |query| {
            query
                .domain_graph_operation(visible_face_neighborhood_operation())
                .traverse(traversal("manager", 2))
                .project(field("identity", "id"))
        },
        |shape| shape.field(result_field("identity", "id", "id")),
    )
}

fn visible_face_neighborhood_operation() -> WorthQueryGraphReadDomainOperationDeclaration {
    WorthQueryGraphReadDomainOperationDeclaration::new(
        "worth.geometry.visible_face_neighborhood",
        1,
        "worth.geometry",
    )
    .expect("operation key should admit")
    .admit_relation_reference("manager")
    .expect("operation reference should admit")
    .requires_support_family("worth.geometry.visible_face_neighborhood.access")
    .expect("support family should admit")
}

fn relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-read-surface-manager",
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
        [SchemaRelationView::new(
            crate::facade::foundation::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            2,
        )],
    )
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
