use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
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
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_portable_type,
    worth_query_principal_binding, worth_query_query_binding, worth_query_relation,
    worth_query_structured_value_binding,
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
                .application_query(principal_query_definition())
                .application_query_binding::<PrincipalQueryBinding>()
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

#[derive(Clone, Copy)]
pub struct PrincipalQueryInput;
pub struct PrincipalQueryParameters;
pub struct PrincipalQueryResult;
struct PrincipalIdentitySlot;

worth_query_structured_value_binding!(pub PrincipalQueryInputBinding for PrincipalQueryInput { identity: "worth.query.tests.principal-query-input.v1" });
worth_query_structured_value_binding!(pub PrincipalQueryParametersBinding for PrincipalQueryParameters { identity: "worth.query.tests.principal-query-parameters.v1" });
worth_query_structured_value_binding!(pub PrincipalQueryResultBinding for PrincipalQueryResult { identity: "worth.query.tests.principal-query-result.v1" });
worth_query_portable_type!(PrincipalIdentitySlot => "worth.query.tests.principal-identity-slot.v1");
worth_query_application_query!(
    pub PrincipalQuery for IdentitySchema,
    identity "worth.query.tests.principal-query.v1",
    parameters PrincipalQueryParametersBinding,
    result PrincipalQueryResultBinding,
    scope Principal => "Principal",
    name "principal_query"
);
worth_query_query_binding!(
    pub PrincipalQueryBinding for PrincipalQueryInput, schema IdentitySchema,
    identity "worth.query.tests.principal-query-binding.v1",
    input PrincipalQueryInputBinding,
    query PrincipalQuery,
    parameters PrincipalQueryParametersBinding => |_| {
        worth_query_declaration::facade::application_query::ApplicationQueryParameterSet::new()
    },
    result PrincipalQueryResultBinding,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Principal, PrincipalIdentity, PrincipalIdentityField, u64,
        ReadOnly,
        worth_query_declaration::facade::application_schema::NoApplicationUnit,
    principal_field PrincipalIdentityField::reference(),
    limits results 8, work 256
);

#[derive(Clone, Copy)]
struct ChangedPrincipalQueryInput;
worth_query_structured_value_binding!(ChangedPrincipalQueryInputBinding for ChangedPrincipalQueryInput { identity: "worth.query.tests.changed-principal-query-input.v1" });
worth_query_query_binding!(
    ChangedPrincipalQueryBinding for ChangedPrincipalQueryInput, schema IdentitySchema,
    identity "worth.query.tests.principal-query-binding.v1",
    input ChangedPrincipalQueryInputBinding,
    query PrincipalQuery,
    parameters PrincipalQueryParametersBinding => |_| {
        worth_query_declaration::facade::application_query::ApplicationQueryParameterSet::new()
    },
    result PrincipalQueryResultBinding,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Principal, PrincipalIdentity, PrincipalIdentityField, u64,
        ReadOnly,
        worth_query_declaration::facade::application_schema::NoApplicationUnit,
    principal_field PrincipalIdentityField::reference(),
    limits results 8, work 257
);

fn principal_query_definition() -> ApplicationQueryDefinition<
    IdentitySchema,
    PrincipalQuery,
    PrincipalQueryParameters,
    PrincipalQueryResult,
    Principal,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentitySchema,
        PrincipalQuery,
        Principal,
        PrincipalQueryResult,
        PrincipalQueryResultBinding,
    >::new(Principal::reference())
    .field(ApplicationQueryResultFieldRef::<
        PrincipalQuery,
        PrincipalIdentitySlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("identity", PrincipalIdentityField::reference()))
    .build();
    ApplicationQueryDefinitionBuilder::declare(PrincipalQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}

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
fn installed_query_binding_retains_exact_principal_scope_and_finite_limits() {
    let schema = installed_index()
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let binding = schema
        .installed_query_binding::<PrincipalQueryBinding>()
        .unwrap();

    assert_eq!(binding.query().name(), "principal_query");
    assert_eq!(binding.principal_binding().binding(), "IdentityBinding");
    assert_eq!(
        binding.scope_contract().field().locus().field(),
        "PrincipalIdentityField"
    );
    assert_eq!(binding.limits().maximum_results().get(), 8);
    assert_eq!(binding.limits().maximum_work().get(), 256);
    assert!(binding
        .limits()
        .narrow(
            std::num::NonZeroUsize::new(1).unwrap(),
            std::num::NonZeroUsize::new(128).unwrap(),
        )
        .is_ok());
    assert_eq!(
        binding
            .limits()
            .narrow(
                std::num::NonZeroUsize::new(9).unwrap(),
                std::num::NonZeroUsize::new(128).unwrap(),
            )
            .unwrap_err(),
        crate::facade::WorthQueryApplicationQueryLimitDenial::MaximumResultsWidened
    );
}

#[test]
fn installed_query_binding_rejects_same_identity_with_changed_input_and_limits() {
    let schema = installed_index()
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();

    assert_eq!(
        schema
            .installed_query_binding::<ChangedPrincipalQueryBinding>()
            .err()
            .unwrap()
            .kind(),
        crate::facade::WorthQueryApplicationQueryInstallationDenialKind::BindingMeaningChanged
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
