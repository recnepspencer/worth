use super::*;

pub(super) fn test_principal_binding<Schema>() -> ApplicationPrincipalBindingRef<
    Schema,
    PrincipalBinding,
    FixtureEntity<Schema>,
    FixtureEntity<Schema>,
    u64,
    worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
>
where
    Schema: ApplicationSchema,
{
    let identity = ApplicationFieldRef::<
        Schema,
        FixtureEntity<Schema>,
        FixtureIdentityAspect<Schema>,
        FixtureExternalIdentityField<Schema>,
        WorthQueryExternalPrincipalIdentity,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    let status = ApplicationFieldRef::<
        Schema,
        FixtureEntity<Schema>,
        FixtureIdentityAspect<Schema>,
        FixtureMappingStatusField<Schema>,
        WorthQueryPrincipalMappingStatus,
        ReadWrite,
        NoEqualityPredicate,
    >::from_schema_types();
    let target = ApplicationRelationRef::<
        Schema,
        MappingTarget,
        FixtureEntity<Schema>,
        FixtureEntity<Schema>,
    >::from_schema_identifiers("MappingTarget", "TestEntity", "TestEntity",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let principal_identity = ApplicationFieldRef::<
        Schema,
        FixtureEntity<Schema>,
        FixtureIdentityAspect<Schema>,
        FixturePrincipalIdentityField<Schema>,
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
