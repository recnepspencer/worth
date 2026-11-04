//! The aftermath fixture's principal binding.
use super::*;

pub fn fixture_principal_binding() -> ApplicationPrincipalBindingRef<
    AftermathFixtureSchema,
    PrincipalBinding,
    FixtureEntity,
    FixtureEntity,
    u64,
    U64ApplicationValueBinding,
> {
    let identity = ApplicationFieldRef::<
        AftermathFixtureSchema,
        FixtureEntity,
        IdentityAspect,
        ExternalIdentityField,
        WorthQueryExternalPrincipalIdentity,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    let status = ApplicationFieldRef::<
        AftermathFixtureSchema,
        FixtureEntity,
        IdentityAspect,
        MappingStatusField,
        WorthQueryPrincipalMappingStatus,
        ReadWrite,
        NoEqualityPredicate,
    >::from_schema_types();
    let target = mapping_target();
    let principal_identity = ApplicationFieldRef::<
        AftermathFixtureSchema,
        FixtureEntity,
        IdentityAspect,
        PrincipalIdentityField,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
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
