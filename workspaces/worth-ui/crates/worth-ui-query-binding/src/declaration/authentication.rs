use super::WorthUiApplicationSchema;
use worth_query_decl::facade::application_schema::{fragment::RecordSelection, *};

pub struct WorthUiPrincipalBinding;
impl WorthUiPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        WorthUiApplicationSchema,
        Self,
        WorthUiExternalPrincipalMapping,
        WorthUiPrincipal,
        <WorthUiPrincipalId as RecordSelection>::Value,
        <WorthUiPrincipalId as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "WorthUiPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    WorthUiExternalIdentityKey::at("WorthUiExternalIdentityKey"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    WorthUiMappingStatus::at("WorthUiMappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    WorthUiExternalPrincipal::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    WorthUiPrincipalId::at("WorthUiPrincipalId"),
                ),
            },
        )
    }
}

pub use super::application_schema::storage_model::{
    WorthUiExternalIdentityKey, WorthUiExternalPrincipal, WorthUiExternalPrincipalMapping,
    WorthUiMappingStatus, WorthUiPrincipal, WorthUiPrincipalId,
};
