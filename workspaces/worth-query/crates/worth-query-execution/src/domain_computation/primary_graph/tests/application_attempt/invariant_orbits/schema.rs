//! Two genuinely authored schema types; publication rebinds the same declaration.
#[path = "schema/records.rs"]
pub(super) mod records;
use super::component_invariants::declaration::Spare;
pub use records::*;
use worth_query_declaration::{
    facade::{application_schema::*, authentication::*},
    *,
};

pub trait OrbitModel: ApplicationSchema {
    fn next_integrity() -> ApplicationRelationIntegrity;
}
#[derive(Clone, Debug)]
pub struct OrbitSchema<const ORBIT: bool>;
impl<const O: bool> OrbitModel for OrbitSchema<O> {
    fn next_integrity() -> ApplicationRelationIntegrity {
        let relation: ApplicationRelationRef<Self, Next<O>, Node<O>, Node<O>> =
            fragment::relation(OrbitRecords::<O>, Node::<O>, Next::<O>);
        relation.integrity()
    }
}
impl<const O: bool> ApplicationSchema for OrbitSchema<O> {
    const OWNER: &'static str = "invariant_orbit_test";
    const NAME: &'static str = "OrbitSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}

pub struct Next<const O: bool = true>;
impl<const O: bool> fragment::Relation for Next<O> {
    const NAME: &'static str = "Next";
    type To = Node<O>;
    const ORBIT: bool = O;
}
impl Next {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers("Next", "Node", "Node", S::next_integrity())
    }
}
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, ExternalMapping, Principal>
    {
        fragment::relation(OrbitRecords::<true>, ExternalMapping, Self)
    }
}
pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference<S: OrbitModel>() -> ApplicationPrincipalBindingRef<
        S,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
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
                    PrincipalKey::at("PrincipalKey"),
                ),
            },
        )
    }
}
#[derive(Clone, Debug)]
pub struct ReplaceNextInput;
worth_query_portable_type!(ReplaceNextInput => "worth.query.test.orbit-swap.v1");
worth_query_structured_value_binding!(pub ReplaceNextInputBinding for ReplaceNextInput { identity: "worth.query.test.orbit-swap.v1" });
worth_query_operation!(pub ReplaceNext for S: OrbitModel, input ReplaceNextInputBinding);
worth_query_operation_reads!(ReplaceNext => [Node, Next, Spare]);
worth_query_operation_creates!(ReplaceNext => [Spare]);
worth_query_operation_deletes!(ReplaceNext => [Spare]);

worth_query_operation_links!(ReplaceNext => [Next]);
worth_query_operation_unlinks!(ReplaceNext => [Next]);

use worth_query_declaration::facade::application_schema::fragment::{self, *};
impl<const O: bool> Fragment for OrbitSchema<O> {
    const NAME: &'static str = "OrbitSchema";
    type Bindings = NoBindings;
    type Members = (OrbitRecords<O>, GrantRecords, OrbitContracts<O>);
}
pub struct OrbitContracts<const O: bool>;
impl<const O: bool> Registration<OrbitSchema<O>> for OrbitContracts<O> {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<OrbitSchema<O>>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<OrbitSchema<O>>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(super::declared_queries::declarations::declare(
            super::decision::binding::declare(super::component_invariants::declaration::declare(
                schema
                    .principal_binding(IdentityBinding::reference())
                    .operation(
                        ReplaceNext::reference()
                            .definition()
                            .no_external_effect()
                            .no_aftermath()
                            .finish(),
                    )
                    .operation_read_entity(ReplaceNext::reference(), Node::reference())
                    .operation_read_field(ReplaceNext::reference(), NodeKey::at("NodeKey"))
                    .operation_expected_fact(ReplaceNext::reference(), NodeKey::at("NodeKey"))
                    .operation_read_relation(ReplaceNext::reference(), Next::reference())
                    .operation_link(ReplaceNext::reference(), Next::reference())
                    .operation_unlink(ReplaceNext::reference(), Next::reference())
                    .invariant(super::probe::definition::<O>()),
            )),
        ))
    }
}
