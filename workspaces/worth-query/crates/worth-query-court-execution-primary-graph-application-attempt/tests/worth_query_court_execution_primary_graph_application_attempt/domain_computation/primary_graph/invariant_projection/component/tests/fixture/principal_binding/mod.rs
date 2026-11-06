//! The application fixture installs and seeds its required real principal binding.
use super::*;

pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
pub(super) struct IdentityBinding;

pub fn target() -> ApplicationRelationRef<Schema, MappingTarget, Mapping, Principal> {
    ApplicationRelationRef::from_schema_identifiers(
        "MappingTarget",
        "Mapping",
        "Principal",
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}
pub fn reference() -> ApplicationPrincipalBindingRef<
    Schema,
    IdentityBinding,
    Mapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
> {
    ApplicationPrincipalBindingRef::from_requirements(
        "IdentityBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                ExternalIdentity::at("ExternalIdentity"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                MappingStatus::at("MappingStatus"),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(target()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                PrincipalKey::at("PrincipalKey"),
            ),
        },
    )
}

pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    builder.principal_binding(reference())
}

pub fn seed(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
) {
    let binding = schema.principal_binding(reference()).unwrap();
    bootstrap
        .bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new("component-principal").unwrap(),
            1,
            WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "alice").unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
}

use crate::domain_computation::primary_graph::invariant_projection::component::tests::fixture::storage_model::{Mapping,Principal,ExternalIdentity,MappingStatus,PrincipalKey};

use worth_query_declaration::facade::application_schema::fragment;
