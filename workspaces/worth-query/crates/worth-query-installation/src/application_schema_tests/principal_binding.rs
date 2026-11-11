use super::*;

pub(super) fn test_principal_binding<Schema>() -> ApplicationPrincipalBindingRef<
    Schema,
    PrincipalBinding,
    FixtureEntity,
    FixtureEntity,
    u64,
    worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
>
where
    Schema: ApplicationSchema,
{
    let identity = worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
        InstallationFields,
        FixtureEntity,
        FixtureIdentityAspect,
        WorthQueryExternalPrincipalIdentity,
    >::at("ExternalIdentityField");
    let status = worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
        InstallationFields,
        FixtureEntity,
        FixtureIdentityAspect,
        WorthQueryPrincipalMappingStatus,
    >::at("MappingStatusField");
    let target = ApplicationRelationRef::<
        Schema,
        MappingTarget,
        FixtureEntity,
        FixtureEntity,
    >::from_schema_identifiers("MappingTarget", "TestEntity", "TestEntity",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let principal_identity =
        worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
            InstallationFields,
            FixtureEntity,
            FixtureIdentityAspect,
            u64,
        >::at("PrincipalIdentityField");
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
