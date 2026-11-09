use worth_query_decl::facade::application_schema::{fragment::RecordSelection, *};

use super::entities::{ExternalPrincipalMapping, Principal};
use super::relations::ExternalPrincipal;
use super::BankSchema;

pub struct BankPrincipalBinding;
impl BankPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        BankSchema,
        Self,
        ExternalPrincipalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "BankPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityKey::at("ExternalIdentityKey"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    ExternalMappingStatus::at("ExternalMappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    ExternalPrincipal::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

pub use crate::schema::entities::storage_model::{
    ExternalIdentityKey, ExternalMappingStatus, ExternalPrincipalIdentity, PrincipalIdentity,
    PrincipalIdentityField,
};
