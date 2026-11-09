//! An owned node group with a traversed path and ordinary edit operations.
#[path = "records.rs"]
mod storage_model;

use worth_query_declaration::facade::application_schema::fragment::{self};
#[derive(Clone, Copy)]
pub struct WalkSchema;
impl Fragment for WalkSchema {
    const NAME: &'static str = "WalkSchema";
    type Bindings = NoBindings;
    type Members = (WalkSchemaRecords, Contracts);
}
impl ApplicationSchema for WalkSchema {
    const OWNER: &'static str = "invariant_walk_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl WalkSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<WalkSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<WalkSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<WalkSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(members(schema))
    }
}

pub fn members(
    schema: ApplicationSchemaDeclarationBuilder<WalkSchema>,
) -> ApplicationSchemaDeclarationBuilder<WalkSchema> {
    let schema = schema
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
        .operation_read_field(ReplaceNext::reference(), NodeKey::at("NodeKey"))
        .operation_expected_fact(ReplaceNext::reference(), NodeKey::at("NodeKey"))
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
        .operation_read_field(InspectWalk::reference(), NodeKey::at("NodeKey"))
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
        .operation_read_field(SetActive::reference(), NodeKey::at("NodeKey"))
        .operation_read_field(SetActive::reference(), Active::at("Active"))
        .operation_expected_fact(SetActive::reference(), Active::at("Active"))
        .operation_write(SetActive::reference(), Active::at("Active"))
        .invariant(crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::rule::definition())
        .invariant(crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::definition());
    crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::reobservation::declaration::declare(crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::declarations::declare(
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::decision_walks::binding::declare(schema),
    ))
}

pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        WalkSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "IdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

worth_query_ability!(pub EditNode scoped_to Node, in WalkSchema);
worth_query_policy!(pub NodeAccess in WalkSchema);

#[derive(Clone, Debug)]
pub struct ReplaceNextInput;
worth_query_portable_type!(ReplaceNextInput => "worth.query.test.replace-walk-edge.v1");
worth_query_structured_value_binding!(pub ReplaceNextInputBinding for ReplaceNextInput { identity: "worth.query.test.replace-walk-edge.v1" });
worth_query_operation!(pub ReplaceNext for WalkSchema, input ReplaceNextInputBinding);
worth_query_operation_requires!(ReplaceNext => [EditNode]);
worth_query_operation_reads!(ReplaceNext => [Node, Next]);

worth_query_operation_links!(ReplaceNext => [Next]);
worth_query_operation_unlinks!(ReplaceNext => [Next]);

worth_query_operation!(pub InspectWalk for WalkSchema, input ReplaceNextInputBinding);
worth_query_operation_requires!(InspectWalk => [EditNode]);
worth_query_operation_reads!(InspectWalk => [Node, Next]);

#[derive(Clone, Debug)]
pub struct SetActiveInput;
worth_query_portable_type!(SetActiveInput => "worth.query.test.activate-walk-root.v1");
worth_query_structured_value_binding!(pub SetActiveInputBinding for SetActiveInput { identity: "worth.query.test.activate-walk-root.v1" });
worth_query_operation!(pub SetActive for WalkSchema, input SetActiveInputBinding);
worth_query_operation_requires!(SetActive => [EditNode]);
worth_query_operation_reads!(SetActive => [Node]);

pub use storage_model::*;
use worth_query_declaration::facade::application_schema::fragment::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_declaration::*;
