use super::*;

struct ForgedPrincipalIdentityField<Value>(std::marker::PhantomData<fn() -> Value>);

pub(super) trait PrincipalIdentityFixtureValue: 'static {
    type Binding: crate::facade::application_schema::ApplicationIdentityScalarValueBinding<
        Value = Self,
    >;
}

impl PrincipalIdentityFixtureValue for u64 {
    type Binding = U64ApplicationValueBinding;
}

impl PrincipalIdentityFixtureValue for String {
    type Binding = StringApplicationValueBinding;
}

impl<Value: PrincipalIdentityFixtureValue>
    crate::facade::application_schema::DeclaredApplicationFieldValue
    for ForgedPrincipalIdentityField<Value>
{
    type Value = Value;
    type Binding = Value::Binding;
    const PRESENCE: crate::facade::application_schema::ApplicationFieldPresence =
        crate::facade::application_schema::ApplicationFieldPresence::Required;
}

pub(super) fn forged_binding<PrincipalIdentityValue>(
    identity_field: &'static str,
    status_field: &'static str,
    target_relation: &'static str,
    principal_identity_field: &'static str,
) -> ApplicationPrincipalBindingRef<
    IdentitySchema,
    IdentityBinding,
    ExternalMapping,
    Principal,
    PrincipalIdentityValue,
    PrincipalIdentityValue::Binding,
>
where
    PrincipalIdentityValue: PrincipalIdentityFixtureValue,
{
    let identity =
        ApplicationFieldRef::<
            IdentitySchema,
            ExternalMapping,
            ExternalIdentity,
            ExternalIdentityField,
            WorthQueryExternalPrincipalIdentity,
            ReadOnly,
            EqualityPredicate,
        >::from_schema_identifiers("ExternalMapping", "ExternalIdentity", identity_field);
    let status = ApplicationFieldRef::<
        IdentitySchema,
        ExternalMapping,
        ExternalIdentity,
        MappingStatusField,
        WorthQueryPrincipalMappingStatus,
        ReadWrite,
        EqualityPredicate,
    >::from_schema_identifiers("ExternalMapping", "ExternalIdentity", status_field);
    let target = ApplicationRelationRef::<
        IdentitySchema,
        MappingTarget,
        ExternalMapping,
        Principal,
    >::from_schema_identifiers(
        target_relation,
        "ExternalMapping",
        "Principal",
        crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    );
    let principal_identity = ApplicationFieldRef::<
        IdentitySchema,
        Principal,
        PrincipalIdentity,
        ForgedPrincipalIdentityField<PrincipalIdentityValue>,
        PrincipalIdentityValue,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_identifiers(
        "Principal", "PrincipalIdentity", principal_identity_field
    );
    ApplicationPrincipalBindingRef::from_requirements(
        "IdentityBinding",
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
