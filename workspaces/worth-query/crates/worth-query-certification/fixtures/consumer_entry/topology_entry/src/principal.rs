use super::{TopologyModelBinding, TopologyRecords};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::application_schema::{
    fragment, ApplicationRelationCardinality, ApplicationRelationDeletionPolicy,
};

pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
    const SELF_EDGES: bool = true;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, None, None, None);
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit;
}
impl MappingTarget {
    pub fn reference<S: TopologyModelBinding>(
    ) -> ApplicationRelationRef<S, Self, ExternalPrincipalMapping, Principal> {
        fragment::relation(TopologyRecords, ExternalPrincipalMapping, Self)
    }
}

pub struct ConsumerPrincipalBinding;
impl ConsumerPrincipalBinding {
    pub fn reference<Schema: TopologyModelBinding>() -> ApplicationPrincipalBindingRef<
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
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference::<Schema>(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

pub use super::storage_model::{
    ExternalIdentityField, ExternalPrincipalMapping, MappingStatusField, Principal,
    PrincipalIdentity,
};
