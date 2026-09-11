use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldMarkerIdentity, ApplicationFieldPresence, ApplicationFieldRef,
    ApplicationPrincipalBindingRef, ApplicationPrincipalBindingRequirements,
    ApplicationPrincipalIdentityRequirement, ApplicationPrincipalMappingIdentityRequirement,
    ApplicationPrincipalMappingStatusRequirement, ApplicationPrincipalTargetRequirement,
    ApplicationRelationRef, DeclaredApplicationFieldValue, EqualityPredicate, ReadOnly, ReadWrite,
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_principal_binding, worth_query_relation,
};

use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPrincipalBindingInstallationDenialKind,
};

worth_query_application_schema! {
    pub schema IdentitySchema {
        owner: identity_installation_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Principal::reference(),
                )
                .principal_binding(IdentityBinding::reference())
        }
    }
}

worth_query_entity!(pub ExternalMapping for IdentitySchema);
worth_query_entity!(pub Principal for IdentitySchema);
worth_query_aspect!(pub ExternalIdentity for IdentitySchema, ExternalMapping; identity = AspectIdentity(0x91611046), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for IdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity for IdentitySchema, Principal; identity = AspectIdentity(0x91611047), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for IdentitySchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatusField for IdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_relation!(pub MappingTarget in IdentitySchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
struct ForgedPrincipalIdentityField;
impl ApplicationFieldMarkerIdentity<IdentitySchema, Principal, PrincipalIdentity>
    for ForgedPrincipalIdentityField
{
    const IDENTIFIER: &'static str = "PrincipalIdentityField";
}
impl DeclaredApplicationFieldValue for ForgedPrincipalIdentityField {
    type Value = String;
    type Binding = StringApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}
worth_query_principal_binding!(
    pub IdentityBinding in IdentitySchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);

#[test]
fn installed_principal_binding_is_runtime_generation_and_schema_affine() {
    let index = installed_index();
    let schema = index
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    assert!(binding.principal_identity_binding().is_identity());
    assert_eq!(
        binding
            .decode_principal_identity(&AspectValue::UInt64(42))
            .unwrap(),
        42
    );
    index.validate_principal_binding(&binding).unwrap();

    let foreign = installed_index();
    assert_eq!(
        foreign
            .validate_principal_binding(&binding)
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalBindingInstallationDenialKind::ForeignRuntime
    );

    let successor = index.successor_generation();
    assert_eq!(
        successor
            .validate_principal_binding(&binding)
            .unwrap_err()
            .kind(),
        WorthQueryPrincipalBindingInstallationDenialKind::StaleGeneration
    );
}

#[test]
fn copied_binding_identifiers_cannot_change_target_principal_identity_type() {
    let index = installed_index();
    let schema = index
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let forged = forged_string_identity_binding();

    assert_eq!(
        schema.principal_binding(forged).unwrap_err().kind(),
        WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged
    );
}

fn forged_string_identity_binding() -> ApplicationPrincipalBindingRef<
    IdentitySchema,
    IdentityBinding,
    ExternalMapping,
    Principal,
    String,
    StringApplicationValueBinding,
> {
    let identity = ApplicationFieldRef::<
        IdentitySchema,
        ExternalMapping,
        ExternalIdentity,
        ExternalIdentityField,
        WorthQueryExternalPrincipalIdentity,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    let status = ApplicationFieldRef::<
        IdentitySchema,
        ExternalMapping,
        ExternalIdentity,
        MappingStatusField,
        WorthQueryPrincipalMappingStatus,
        ReadWrite,
        EqualityPredicate,
    >::from_schema_types();
    let target = ApplicationRelationRef::<
        IdentitySchema,
        MappingTarget,
        ExternalMapping,
        Principal,
    >::from_schema_identifiers("MappingTarget", "ExternalMapping", "Principal",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let principal_identity = ApplicationFieldRef::<
        IdentitySchema,
        Principal,
        PrincipalIdentity,
        ForgedPrincipalIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
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

fn installed_index() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "identity_installation_test",
        1,
        0,
    ))
    .application_schema(IdentitySchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
}
