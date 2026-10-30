//! The real schema declaration includes its own typed Source, Index and Parent.
use super::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_declaration::{worth_query_aspect, worth_query_entity, worth_query_field};

pub trait ModelSchema: ApplicationSchema {
    const MODE: u8;
}
pub struct ParentSchema<const MODE: u8>;
impl<const MODE: u8> ModelSchema for ParentSchema<MODE> {
    const MODE: u8 = MODE;
}
impl<const MODE: u8> ApplicationSchema for ParentSchema<MODE> {
    const OWNER: &'static str = "worth.query.tests.managed-parent";
    const NAME: &'static str = "ParentSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let policy = policy::<Self>();
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(Mapping::reference())
            .entity(Principal::reference())
            .aspect(Mapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .field(Mapping::reference(), ExternalIdentityField::reference())
            .field(Mapping::reference(), MappingStatus::reference())
            .field(Principal::reference(), PrincipalIdentity::reference())
            .relation(
                MappingTarget::reference(),
                Mapping::reference(),
                Principal::reference(),
            )
            .principal_binding(principal_binding::<Self>())
            .policy(policy)
            .ability(ability::<Self>())
            .ability_policy(
                ability::<Self>(),
                policy,
                [ApplicationAuthorizationPathBuilder::from_principal(
                    Principal::reference::<Self>(),
                )
                .allow(Principal::reference::<Self>())],
            )
            .application_query(queries::source::<Self>())
            .application_query_binding::<queries::SourceBinding<Self>>()
            .application_query(queries::parent::<Self>())
            .application_query_binding::<queries::ParentBinding<Self>>()
            .managed_index(ApplicationManagedIndexDefinition::<
                Self,
                queries::ModelIndex<Self>,
            >::new())
            .build()
    }
}
worth_query_entity!(pub Mapping for Schema: ModelSchema);
worth_query_entity!(pub Principal for Schema: ModelSchema);
worth_query_aspect!(pub ExternalIdentity for Schema: ModelSchema, Mapping; identity = AspectIdentity(0x91902001), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Schema: ModelSchema, Principal; identity = AspectIdentity(0x91902002), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for Schema: ModelSchema, Mapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatus for Schema: ModelSchema, Mapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentity for Schema: ModelSchema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
pub struct MappingTarget;
impl MappingTarget {
    pub fn reference<S: ModelSchema>() -> ApplicationRelationRef<S, Self, Mapping, Principal> {
        ApplicationRelationRef::from_schema_identifiers(
            "MappingTarget",
            "Mapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}

pub struct PrincipalBinding;
pub fn principal_binding<S: ModelSchema>() -> ApplicationPrincipalBindingRef<
    S,
    PrincipalBinding,
    Mapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
> {
    ApplicationPrincipalBindingRef::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                ExternalIdentityField::reference::<S>(),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                MappingStatus::reference::<S>(),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(
                MappingTarget::reference::<S>(),
            ),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                PrincipalIdentity::reference::<S>(),
            ),
        },
    )
}
pub struct CloseAssessment;
pub struct ManagedParentPolicy;
pub fn ability<S: ModelSchema>() -> ApplicationAbilityRef<S, CloseAssessment, Principal> {
    ApplicationAbilityRef::from_schema_identifiers("CloseAssessment", "Principal")
}
pub fn policy<S: ModelSchema>() -> ApplicationPolicyRef<S, ManagedParentPolicy> {
    ApplicationPolicyRef::from_schema_identifier("ManagedParentPolicy")
}
