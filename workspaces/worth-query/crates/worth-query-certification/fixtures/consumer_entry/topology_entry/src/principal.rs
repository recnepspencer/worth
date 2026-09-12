use super::TopologySchemaBinding;
use worth_query_decl::facade::{
    application_schema::*, authentication::*, worth_query_aspect, worth_query_entity,
    worth_query_field,
};

worth_query_entity!(pub ExternalPrincipalMapping for Schema: TopologySchemaBinding);
worth_query_entity!(pub Principal for Schema: TopologySchemaBinding);
worth_query_aspect!(pub ExternalIdentity for Schema: TopologySchemaBinding, ExternalPrincipalMapping; identity = AspectIdentity(0x9174_1021), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for Schema: TopologySchemaBinding, Principal; identity = AspectIdentity(0x9174_1022), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for Schema: TopologySchemaBinding, ExternalPrincipalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for Schema: TopologySchemaBinding, ExternalPrincipalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentity for Schema: TopologySchemaBinding, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);

pub struct MappingTarget;
impl MappingTarget {
    pub fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, ExternalPrincipalMapping, Principal> {
        ApplicationRelationRef::from_schema_identifiers(
            "MappingTarget",
            "ExternalPrincipalMapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}

pub struct ConsumerPrincipalBinding;
impl ConsumerPrincipalBinding {
    pub fn reference<Schema: TopologySchemaBinding>() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "ConsumerPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::reference::<Schema>(),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::reference::<Schema>(),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference::<Schema>(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::reference::<Schema>(),
                ),
            },
        )
    }
}
