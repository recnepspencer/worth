//! The application fixture installs and seeds its required real principal binding.
use super::*;
use worth_query_declaration::facade::authentication::*;

worth_query_entity!(Mapping for Schema);
worth_query_entity!(Principal for Schema);
worth_query_aspect!(MappingData for Schema, Mapping; identity = AspectIdentity(0x9181_5002), revision = AspectContractRevision(1));
worth_query_field!(ExternalIdentity for Schema, Mapping, MappingData: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(MappingStatus for Schema, Mapping, MappingData: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_aspect!(PrincipalData for Schema, Principal; identity = AspectIdentity(0x9181_5003), revision = AspectContractRevision(1));
worth_query_field!(PrincipalKey for Schema, Principal, PrincipalData: u64 => U64ApplicationValueBinding, read_only, equality);
struct MappingTarget;
struct IdentityBinding;

fn target() -> ApplicationRelationRef<Schema, MappingTarget, Mapping, Principal> {
    ApplicationRelationRef::from_schema_identifiers(
        "MappingTarget",
        "Mapping",
        "Principal",
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    )
}
fn reference() -> ApplicationPrincipalBindingRef<
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
                ExternalIdentity::reference(),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                MappingStatus::reference(),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(target()),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                PrincipalKey::reference(),
            ),
        },
    )
}

pub(super) fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    builder
        .entity(Mapping::reference())
        .entity(Principal::reference())
        .aspect(Mapping::reference(), MappingData::reference())
        .field(Mapping::reference(), ExternalIdentity::reference())
        .field(Mapping::reference(), MappingStatus::reference())
        .aspect(Principal::reference(), PrincipalData::reference())
        .field(Principal::reference(), PrincipalKey::reference())
        .relation(target(), Mapping::reference(), Principal::reference())
        .principal_binding(reference())
}

pub(super) fn seed(
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
