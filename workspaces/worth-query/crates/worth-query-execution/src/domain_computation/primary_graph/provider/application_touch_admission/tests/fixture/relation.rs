//! Actual relation contracts; Link-only and initialized-create grants differ explicitly.
use super::*;
pub(super) struct Edge;
impl Edge {
    pub const fn reference() -> ApplicationRelationRef<TouchSchema, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers(
            "Edge",
            "Node",
            "Node",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
worth_query_aspect!(pub(super) Payload for TouchSchema, Edge;
    identity = AspectIdentity(0x91811002), revision = AspectContractRevision(1));
worth_query_decl::facade::worth_query_relation_field!(pub(super) Weight for TouchSchema,
    Edge, Node, Node, Payload: u64 => U64ApplicationValueBinding, read_write);
worth_query_operation!(pub(in crate::domain_computation::primary_graph::provider::application_touch_admission::tests) LinkOnly for TouchSchema, input InputBinding);
worth_query_operation!(pub(in crate::domain_computation::primary_graph::provider::application_touch_admission::tests) Initialize for TouchSchema, input InputBinding);
worth_query_operation_reads!(LinkOnly => [Node]);
worth_query_operation_reads!(Initialize => [Node]);
worth_query_operation_writes!(Initialize => [Weight]);
impl OperationLinks<LinkOnly> for Edge {}
impl OperationLinks<Initialize> for Edge {}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<TouchSchema>,
) -> ApplicationSchemaDeclarationBuilder<TouchSchema> {
    let link = LinkOnly::reference();
    let init = Initialize::reference();
    schema
        .relation(Edge::reference(), Node::reference(), Node::reference())
        .relation_aspect(
            ApplicationRelationAspectDefinition::new(
                Edge::reference(),
                Payload::reference(),
                worth_foundational::facade::AbsenceLaw::Optional,
            )
            .field(Weight::reference())
            .finish()
            .unwrap(),
        )
        .operation(
            link.definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(link, 16)
        .operation_read_entity(link, Node::reference())
        .operation_link(link, Edge::reference())
        .operation(
            init.definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(init, 16)
        .operation_read_entity(init, Node::reference())
        .operation_link(init, Edge::reference())
        .operation_initialize_relation_field(init, Weight::reference())
}
