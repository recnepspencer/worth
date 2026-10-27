//! An owned node group with a traversed path and ordinary edit operations.
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_declaration::*;

worth_query_application_schema! {
    pub schema WalkSchema {
        owner: invariant_walk_test,
        version: (1, 0),
        members: |schema| {
            members(schema)
        }
    }
}

fn members(
    schema: ApplicationSchemaDeclarationBuilder<WalkSchema>,
) -> ApplicationSchemaDeclarationBuilder<WalkSchema> {
    let schema = schema
        .entity(ExternalMapping::reference())
        .entity(Principal::reference())
        .entity(Node::reference())
        .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
        .field(
            ExternalMapping::reference(),
            ExternalIdentityField::reference(),
        )
        .field(
            ExternalMapping::reference(),
            MappingStatusField::reference(),
        )
        .aspect(Principal::reference(), PrincipalIdentity::reference())
        .field(Principal::reference(), PrincipalIdentityField::reference())
        .aspect(Node::reference(), NodeData::reference())
        .field(Node::reference(), NodeKey::reference())
        .field(Node::reference(), Active::reference())
        .field(Node::reference(), Blocked::reference())
        .relation(
            MappingTarget::reference(),
            ExternalMapping::reference(),
            Principal::reference(),
        )
        .relation(
            NodeOwner::reference(),
            Principal::reference(),
            Node::reference(),
        )
        .relation(GroupRoot::reference(), Node::reference(), Node::reference())
        .relation(Next::reference(), Node::reference(), Node::reference())
        .principal_binding(IdentityBinding::reference())
        .ability(EditNode::reference())
        .policy(NodeAccess::reference())
        .ability_policy(
            EditNode::reference(),
            NodeAccess::reference(),
            [
                ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                    .forward(NodeOwner::reference())
                    .allow(Node::reference()),
            ],
        )
        .operation(
            ReplaceNext::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(ReplaceNext::reference(), 128)
        .operation_requires_ability(ReplaceNext::reference(), EditNode::reference())
        .operation_read_entity(ReplaceNext::reference(), Node::reference())
        .operation_read_field(ReplaceNext::reference(), NodeKey::reference())
        .operation_expected_fact(ReplaceNext::reference(), NodeKey::reference())
        .operation_read_relation(ReplaceNext::reference(), Next::reference())
        .operation_link(ReplaceNext::reference(), Next::reference())
        .operation_unlink(ReplaceNext::reference(), Next::reference())
        .operation(
            InspectWalk::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_requires_ability(InspectWalk::reference(), EditNode::reference())
        .operation_read_entity(InspectWalk::reference(), Node::reference())
        .operation_read_field(InspectWalk::reference(), NodeKey::reference())
        .operation_read_relation(InspectWalk::reference(), Next::reference())
        .operation(
            SetActive::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(SetActive::reference(), 64)
        .operation_requires_ability(SetActive::reference(), EditNode::reference())
        .operation_read_entity(SetActive::reference(), Node::reference())
        .operation_read_field(SetActive::reference(), NodeKey::reference())
        .operation_read_field(SetActive::reference(), Active::reference())
        .operation_expected_fact(SetActive::reference(), Active::reference())
        .operation_write(SetActive::reference(), Active::reference())
        .invariant(super::rule::definition())
        .invariant(super::probe::definition());
    super::declared_queries::declarations::declare(super::decision_walks::binding::declare(schema))
}

worth_query_entity!(pub ExternalMapping for WalkSchema);
worth_query_entity!(pub Principal for WalkSchema);
worth_query_entity!(pub Node for WalkSchema);
worth_query_aspect!(pub ExternalIdentity for WalkSchema, ExternalMapping; identity = AspectIdentity(0x9181_3101), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for WalkSchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for WalkSchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_aspect!(pub PrincipalIdentity for WalkSchema, Principal; identity = AspectIdentity(0x9181_3102), revision = AspectContractRevision(1),);
worth_query_field!(pub PrincipalIdentityField for WalkSchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub MappingTarget in WalkSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub IdentityBinding in WalkSchema,
    mapping ExternalMapping { identity: ExternalIdentityField, status: MappingStatusField,
        target: MappingTarget => Principal, principal_identity: PrincipalIdentityField });

worth_query_aspect!(pub NodeData for WalkSchema, Node; identity = AspectIdentity(0x9181_3103), revision = AspectContractRevision(1),);
worth_query_field!(pub NodeKey for WalkSchema, Node, NodeData: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Active for WalkSchema, Node, NodeData: bool => BoolApplicationValueBinding, read_write, equality);
worth_query_field!(pub Blocked for WalkSchema, Node, NodeData: bool => BoolApplicationValueBinding, read_write, equality);
worth_query_relation!(pub NodeOwner in WalkSchema, Principal => Node; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub GroupRoot in WalkSchema, Node => Node; integrity = ApplicationRelationIntegrity::new(
    ApplicationRelationEndpoints::same_context(true),
    ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None),
    ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
));
worth_query_relation!(pub Next in WalkSchema, Node => Node; integrity = same_context_unbounded_retain_dangling);
worth_query_ability!(pub EditNode scoped_to Node, in WalkSchema);
worth_query_policy!(pub NodeAccess in WalkSchema);

#[derive(Clone, Debug)]
pub struct ReplaceNextInput;
worth_query_portable_type!(ReplaceNextInput => "worth.query.test.replace-walk-edge.v1");
worth_query_structured_value_binding!(pub ReplaceNextInputBinding for ReplaceNextInput { identity: "worth.query.test.replace-walk-edge.v1" });
worth_query_operation!(pub ReplaceNext for WalkSchema, input ReplaceNextInputBinding);
worth_query_operation_requires!(ReplaceNext => [EditNode]);
worth_query_operation_reads!(ReplaceNext => [Node, NodeKey, Next]);
worth_query_operation_expects_fact!(ReplaceNext => [NodeKey]);
worth_query_operation_links!(ReplaceNext => [Next]);
worth_query_operation_unlinks!(ReplaceNext => [Next]);

worth_query_operation!(pub InspectWalk for WalkSchema, input ReplaceNextInputBinding);
worth_query_operation_requires!(InspectWalk => [EditNode]);
worth_query_operation_reads!(InspectWalk => [Node, NodeKey, Next]);

#[derive(Clone, Debug)]
pub struct SetActiveInput;
worth_query_portable_type!(SetActiveInput => "worth.query.test.activate-walk-root.v1");
worth_query_structured_value_binding!(pub SetActiveInputBinding for SetActiveInput { identity: "worth.query.test.activate-walk-root.v1" });
worth_query_operation!(pub SetActive for WalkSchema, input SetActiveInputBinding);
worth_query_operation_requires!(SetActive => [EditNode]);
worth_query_operation_reads!(SetActive => [Node, NodeKey, Active]);
worth_query_operation_expects_fact!(SetActive => [Active]);
worth_query_operation_writes!(SetActive => [Active]);
