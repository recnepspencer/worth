use super::*;
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
        ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationFieldMarkerIdentity,
        ApplicationFieldPresence, ApplicationFieldRef, ApplicationPrincipalBindingRef,
        ApplicationPrincipalBindingRequirements, ApplicationPrincipalIdentityRequirement,
        ApplicationPrincipalMappingIdentityRequirement,
        ApplicationPrincipalMappingStatusRequirement, ApplicationPrincipalTargetRequirement,
        ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
        ApplicationSchemaDeclarationDenial, DeclaredApplicationFieldValue, EqualityPredicate,
        NoEqualityPredicate, ReadOnly, ReadWrite, U64ApplicationValueBinding,
    },
    authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
        WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
    },
};

struct OtherCapability;
struct PrincipalFacts;
struct MapsToPrincipal;
struct PrincipalBinding;
struct MappingIdentity;
struct MappingStatus;
struct PrincipalIdentity;

worth_query_declaration::worth_query_portable_type!(
    OtherCapability => "worth.query.installation-test.other-capability.v1"
);

impl ApplicationCapabilityMarkerIdentity for OtherCapability {
    type Schema = Schema;
    const IDENTIFIER: &'static str = "Capability";
}

impl ApplicationAspectMarkerIdentity<Schema, Principal> for PrincipalFacts {
    const IDENTIFIER: &'static str = "PrincipalFacts";
    const ASPECT_IDENTITY: worth_query_declaration::facade::application_schema::AspectIdentity =
        worth_query_declaration::facade::application_schema::AspectIdentity(0x9161_2203);
    const CONTRACT_REVISION:
        worth_query_declaration::facade::application_schema::AspectContractRevision =
        worth_query_declaration::facade::application_schema::AspectContractRevision(1);
}

macro_rules! principal_field {
    ($field:ty, $entity:ty, $aspect:ty, $identifier:literal, $value:ty, $binding:ty) => {
        impl ApplicationFieldMarkerIdentity<Schema, $entity, $aspect> for $field {
            const IDENTIFIER: &'static str = $identifier;
        }
        impl DeclaredApplicationFieldValue for $field {
            type Value = $value;
            type Binding = $binding;
            const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
        }
    };
}

principal_field!(
    MappingIdentity,
    Grant,
    Facts,
    "MappingIdentity",
    WorthQueryExternalPrincipalIdentity,
    WorthQueryExternalPrincipalIdentityBinding
);
principal_field!(
    MappingStatus,
    Grant,
    Facts,
    "MappingStatus",
    WorthQueryPrincipalMappingStatus,
    WorthQueryPrincipalMappingStatusBinding
);
principal_field!(
    PrincipalIdentity,
    Principal,
    PrincipalFacts,
    "PrincipalIdentity",
    u64,
    U64ApplicationValueBinding
);

fn principal_binding() -> ApplicationPrincipalBindingRef<
    Schema,
    PrincipalBinding,
    Grant,
    Principal,
    u64,
    U64ApplicationValueBinding,
> {
    let mapping_identity = ApplicationFieldRef::<
        Schema,
        Grant,
        Facts,
        MappingIdentity,
        WorthQueryExternalPrincipalIdentity,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    let mapping_status = ApplicationFieldRef::<
        Schema,
        Grant,
        Facts,
        MappingStatus,
        WorthQueryPrincipalMappingStatus,
        ReadWrite,
        NoEqualityPredicate,
    >::from_schema_types();
    let target = ApplicationRelationRef::<Schema, MapsToPrincipal, Grant, Principal>::
        from_schema_identifiers("MapsToPrincipal", "Grant", "Principal",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let principal_identity = ApplicationFieldRef::<
        Schema,
        Principal,
        PrincipalFacts,
        PrincipalIdentity,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
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
        let grant = ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant");
        let resource = ApplicationEntityRef::<Schema, Resource>::from_schema_identifier("Resource");
        let principal =
            ApplicationEntityRef::<Schema, Principal>::from_schema_identifier("Principal");
        let operation = ApplicationOperationRef::<Schema, Operation, ()>::from_declaration();
        ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
            .entity(grant)
            .entity(resource)
            .entity(principal)
            .aspect(
                grant,
                ApplicationAspectRef::<Schema, Grant, Facts>::from_schema_identifier("Facts"),
            )
            .aspect(
                resource,
                ApplicationAspectRef::<Schema, Resource, ResourceFacts>::from_schema_identifier(
                    "ResourceFacts",
                ),
            )
            .aspect(
                principal,
                ApplicationAspectRef::<Schema, Principal, PrincipalFacts>::from_schema_identifier(
                    "PrincipalFacts",
                ),
            )
            .field(grant, field::<Action>())
            .field(grant, field::<Purpose>())
            .field(grant, field::<Field>())
            .field(grant, field::<Amount>())
            .field(grant, field::<Workflow>())
            .field(grant, field::<Status>())
            .field(grant, field::<ValidFrom>())
            .field(grant, field::<ValidThrough>())
            .field(grant, field::<DelegationLimit>())
            .field(resource, resource_field::<ResourceWorkflow>())
            .field(grant, mapping_identity_field())
            .field(grant, mapping_status_field())
            .field(principal, principal_identity_field())
            .relation(
                ApplicationRelationRef::<Schema, ResourceRelation, Grant, Resource>::
                    from_schema_identifiers("ResourceRelation", "Grant", "Resource",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
                grant,
                resource,
            )
            .relation(
                ApplicationRelationRef::<Schema, ScopedRelation, Grant, Resource>::
                    from_schema_identifiers("ScopedRelation", "Grant", "Resource",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
                grant,
                resource,
            )
            .relation(
                ApplicationRelationRef::<Schema, PrincipalResource, Principal, Resource>::
                    from_schema_identifiers("PrincipalResource", "Principal", "Resource",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
                principal,
                resource,
            )
            .relation(
                ApplicationRelationRef::<Schema, Parent, Grant, Grant>::from_schema_identifiers(
                    "Parent", "Grant", "Grant",
                    worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
                ),
                grant,
                grant,
            )
            .relation(
                ApplicationRelationRef::<Schema, Grantor, Principal, Grant>::
                    from_schema_identifiers("Grantor", "Principal", "Grant",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
                principal,
                grant,
            )
            .relation(
                ApplicationRelationRef::<Schema, Grantee, Principal, Grant>::
                    from_schema_identifiers("Grantee", "Principal", "Grant",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
                principal,
                grant,
            )
            .relation(
                ApplicationRelationRef::<Schema, MapsToPrincipal, Grant, Principal>::
                    from_schema_identifiers("MapsToPrincipal", "Grant", "Principal",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
                grant,
                principal,
            )
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
            .build()
    }
}

fn mapping_identity_field() -> ApplicationFieldRef<
    Schema,
    Grant,
    Facts,
    MappingIdentity,
    WorthQueryExternalPrincipalIdentity,
    ReadOnly,
    EqualityPredicate,
> {
    ApplicationFieldRef::from_schema_types()
}

fn mapping_status_field() -> ApplicationFieldRef<
    Schema,
    Grant,
    Facts,
    MappingStatus,
    WorthQueryPrincipalMappingStatus,
    ReadWrite,
    NoEqualityPredicate,
> {
    ApplicationFieldRef::from_schema_types()
}

fn principal_identity_field() -> ApplicationFieldRef<
    Schema,
    Principal,
    PrincipalFacts,
    PrincipalIdentity,
    u64,
    ReadOnly,
    EqualityPredicate,
> {
    ApplicationFieldRef::from_schema_types()
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
