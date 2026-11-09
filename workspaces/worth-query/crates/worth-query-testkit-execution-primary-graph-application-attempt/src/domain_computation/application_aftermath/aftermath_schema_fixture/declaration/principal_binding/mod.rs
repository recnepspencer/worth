//! Native principal fields retain their exact declared bindings.
use super::*;
pub fn fixture_principal_binding() -> ApplicationPrincipalBindingRef<
    AftermathFixtureSchema,
    PrincipalBinding,
    FixtureEntity,
    FixtureEntity,
    u64,
    U64ApplicationValueBinding,
> {
    let identity = ExternalIdentityField::at("ExternalIdentityField");
    let status = MappingStatusField::at("MappingStatusField");
    let target = mapping_target();
    let principal_identity = PrincipalIdentityField::at("PrincipalIdentityField");
    ApplicationPrincipalBindingRef::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(identity),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(status),
            target: ApplicationPrincipalTargetRequirement::from_relation(target),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                principal_identity,
            ),
        },
    )
}
