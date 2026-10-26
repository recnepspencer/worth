//! Two genuinely authored schema types; publication rebinds the same declaration.
use worth_query_declaration::{
    facade::{application_schema::*, authentication::*},
    *,
};

pub trait OrbitModel: ApplicationSchema {
    const NEXT: ApplicationRelationIntegrity;
}
#[derive(Clone, Debug)]
pub struct OrbitSchema<const ORBIT: bool>;
impl<const O: bool> OrbitModel for OrbitSchema<O> {
    const NEXT: ApplicationRelationIntegrity = if O {
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit()
    } else {
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
    };
}
impl<const O: bool> ApplicationSchema for OrbitSchema<O> {
    const OWNER: &'static str = "invariant_orbit_test";
    const NAME: &'static str = "OrbitSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        super::decision::binding::declare(
            ApplicationSchemaDeclarationBuilder::for_schema()
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
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .field(Principal::reference(), PrincipalKey::reference())
                .aspect(Node::reference(), NodeData::reference())
                .field(Node::reference(), NodeKey::reference())
                .field(Node::reference(), Marked::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Principal::reference(),
                )
                .principal_binding(IdentityBinding::reference())
                .relation(Next::reference(), Node::reference(), Node::reference())
                .operation(
                    ReplaceNext::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_read_entity(ReplaceNext::reference(), Node::reference())
                .operation_read_field(ReplaceNext::reference(), NodeKey::reference())
                .operation_expected_fact(ReplaceNext::reference(), NodeKey::reference())
                .operation_read_relation(ReplaceNext::reference(), Next::reference())
                .operation_link(ReplaceNext::reference(), Next::reference())
                .operation_unlink(ReplaceNext::reference(), Next::reference())
                .invariant(super::probe::definition::<O>()),
        )
        .build()
    }
}
worth_query_entity!(pub ExternalMapping for S: OrbitModel);
worth_query_entity!(pub Principal for S: OrbitModel);
worth_query_entity!(pub Node for S: OrbitModel);
worth_query_aspect!(pub ExternalIdentity for S: OrbitModel, ExternalMapping; identity = AspectIdentity(0x9181_4101), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for S: OrbitModel, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for S: OrbitModel, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_aspect!(pub PrincipalFacts for S: OrbitModel, Principal; identity = AspectIdentity(0x9181_4102), revision = AspectContractRevision(1));
worth_query_field!(pub PrincipalKey for S: OrbitModel, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_aspect!(pub NodeData for S: OrbitModel, Node; identity = AspectIdentity(0x9181_4103), revision = AspectContractRevision(1));
worth_query_field!(pub NodeKey for S: OrbitModel, Node, NodeData: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Marked for S: OrbitModel, Node, NodeData: bool => BoolApplicationValueBinding, read_write, equality);
pub struct Next;
impl Next {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, Node, Node> {
        ApplicationRelationRef::from_schema_identifiers("Next", "Node", "Node", S::NEXT)
    }
}
pub struct MappingTarget;
impl MappingTarget {
    pub fn reference<S: OrbitModel>() -> ApplicationRelationRef<S, Self, ExternalMapping, Principal>
    {
        ApplicationRelationRef::from_schema_identifiers(
            "MappingTarget",
            "ExternalMapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
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
                    ExternalIdentityField::reference(),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::reference(),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalKey::reference(),
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
worth_query_operation_reads!(ReplaceNext => [Node, NodeKey, Next]);
worth_query_operation_expects_fact!(ReplaceNext => [NodeKey]);
worth_query_operation_links!(ReplaceNext => [Next]);
worth_query_operation_unlinks!(ReplaceNext => [Next]);
