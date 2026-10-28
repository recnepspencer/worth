//! Genuine CRUD grants for observing a proposal's new and removed identities.
use super::super::schema::*;
use worth_query_declaration::{facade::application_schema::*, *};

worth_query_operation!(pub(super) ChangeNode for WalkSchema, input ReplaceNextInputBinding);
worth_query_operation_requires!(ChangeNode => [EditNode]);
worth_query_operation_reads!(ChangeNode => [Node, NodeKey, GroupRoot]);
worth_query_operation_expects_fact!(ChangeNode => [NodeKey]);
worth_query_operation_creates!(ChangeNode => [Node]);
worth_query_operation_deletes!(ChangeNode => [Node]);
worth_query_operation_writes!(ChangeNode => [NodeKey, Active, Blocked]);
worth_query_operation_links!(ChangeNode => [GroupRoot]);
worth_query_operation_unlinks!(ChangeNode => [GroupRoot]);

pub(in super::super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<WalkSchema>,
) -> ApplicationSchemaDeclarationBuilder<WalkSchema> {
    schema
        .operation(
            ChangeNode::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_requires_ability(ChangeNode::reference(), EditNode::reference())
        .operation_read_entity(ChangeNode::reference(), Node::reference())
        .operation_read_field(ChangeNode::reference(), NodeKey::reference())
        .operation_expected_fact(ChangeNode::reference(), NodeKey::reference())
        .operation_read_relation(ChangeNode::reference(), GroupRoot::reference())
        .operation_create(ChangeNode::reference(), Node::reference())
        .operation_delete(ChangeNode::reference(), Node::reference())
        .operation_write(ChangeNode::reference(), NodeKey::reference())
        .operation_write(ChangeNode::reference(), Active::reference())
        .operation_write(ChangeNode::reference(), Blocked::reference())
        .operation_link(ChangeNode::reference(), GroupRoot::reference())
        .operation_unlink(ChangeNode::reference(), GroupRoot::reference())
}
