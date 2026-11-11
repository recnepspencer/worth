use super::*;
mod resource_access;
use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityContextRef, ApplicationCapabilityMarkerIdentity,
        ApplicationCapabilityProvenanceRef, ApplicationCapabilityRef,
    },
    application_schema::{
        ApplicationPrincipalBindingRef, ApplicationPrincipalBindingRequirements,
        ApplicationPrincipalIdentityRequirement, ApplicationPrincipalMappingIdentityRequirement,
        ApplicationPrincipalMappingStatusRequirement, ApplicationPrincipalTargetRequirement,
        ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
        ApplicationSchemaDeclarationDenial, U64ApplicationValueBinding,
    },
};

struct OtherCapability;
struct PrincipalBinding;

worth_query_declaration::worth_query_portable_type!(
    OtherCapability => "worth.query.installation-test.other-capability.v1"
);

impl ApplicationCapabilityMarkerIdentity for OtherCapability {
    type Schema = Schema;
    const IDENTIFIER: &'static str = "Capability";
}

fn principal_binding() -> ApplicationPrincipalBindingRef<
    Schema,
    PrincipalBinding,
    Grant,
    Principal,
    u64,
    U64ApplicationValueBinding,
> {
    let mapping_identity = MappingIdentity::at("MappingIdentity");
    let mapping_status = MappingStatus::at("MappingStatus");
    let target = ApplicationRelationRef::<Schema, MapsToPrincipal, Grant, Principal>::
        from_schema_identifiers("MapsToPrincipal", "Grant", "Principal",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let principal_identity = PrincipalIdentity::at("PrincipalIdentity");
    ApplicationPrincipalBindingRef::from_requirements(
        "PrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                mapping_identity,
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                mapping_status,
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(target),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                principal_identity,
            ),
        },
    )
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth-query-tests";
    const NAME: &'static str = "capability-warm-lookup";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let operation =
            worth_query_declaration::facade::application_schema::ApplicationOperationRef::<
                Schema,
                Operation,
                (),
            >::from_declaration();
        ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(CapabilityFields)
            .unwrap()
            .principal_binding(principal_binding())
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .capability_context(
                ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
            )
            .capability_provenance(
                ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
            )
            .capability(typed_contract())
            .application_query(resource_access::query())
            .application_query_binding::<resource_access::Access>()
            .application_resource(resource_access::definition())
            .build()
    }
}

fn principal_identity_field() -> PrincipalIdentity {
    PrincipalIdentity::at("PrincipalIdentity")
}

#[test]
fn public_warm_lookup_is_one_probe_without_canonical_work_and_rejects_identity_drift() {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth-query-tests",
        1,
        0,
    ))
    .application_schema(Schema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    let installed = index
        .bind_application_schema(Schema::declaration().unwrap())
        .unwrap();
    let operation = ApplicationOperationRef::<Schema, Operation, ()>::from_declaration();
    let capability = installed
        .capability(
            ApplicationCapabilityRef::<Schema, Capability>::from_declaration(),
            operation,
        )
        .unwrap();
    let evidence = capability.lookup_evidence();
    assert_eq!(evidence.registry_probes(), 1);
    assert_eq!(evidence.basis_preparations(), 0);
    assert_eq!(evidence.digest_derivations(), 0);
    assert_eq!(evidence.digest_text_materializations(), 0);

    let denial = match installed.capability(
        ApplicationCapabilityRef::<Schema, OtherCapability>::from_declaration(),
        operation,
    ) {
        Ok(_) => panic!("same-name portable identity drift must miss the registry"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        crate::application_capability::WorthQueryApplicationCapabilityInstallationDenialKind::CapabilityMeaningChanged
    );
}
