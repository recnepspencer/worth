//! R8.2: pre-image demand must be covered by the operation's own decision reads.

use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredRecordedInverse,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldMarkerIdentity, ApplicationFieldPresence, ApplicationOperationRef,
    ApplicationSchema, ApplicationSchemaDeclaration, DeclaredApplicationFieldValue,
    U64ApplicationValueBinding,
};

use super::*;

struct CoveredAftermathSchema;
struct UncoveredAftermathSchema;
struct UncoveredSecretField<Schema>(std::marker::PhantomData<fn() -> Schema>);

impl<Schema>
    ApplicationFieldMarkerIdentity<Schema, FixtureEntity<Schema>, FixtureIdentityAspect<Schema>>
    for UncoveredSecretField<Schema>
{
    const IDENTIFIER: &'static str = "UncoveredSecretField";
}

impl<Schema> DeclaredApplicationFieldValue for UncoveredSecretField<Schema> {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}

impl ApplicationSchema for CoveredAftermathSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "CoveredAftermathSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        aftermath_schema_members::<Self, FixturePrincipalIdentityField<Self>>().build()
    }
}

impl ApplicationSchema for UncoveredAftermathSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "UncoveredAftermathSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        aftermath_schema_members::<Self, UncoveredSecretField<Self>>().build()
    }
}

fn aftermath_schema_members<Schema, PreImageField>() -> ApplicationSchemaDeclarationBuilder<Schema>
where
    Schema: ApplicationSchema,
    PreImageField: ApplicationFieldMarkerIdentity<
        Schema,
        FixtureEntity<Schema>,
        FixtureIdentityAspect<Schema>,
        Value = u64,
    >,
{
    let preimage_locus = DeclaredPreImageLocus::from_field(ApplicationFieldRef::<
        Schema,
        FixtureEntity<Schema>,
        FixtureIdentityAspect<Schema>,
        PreImageField,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types());
    test_schema_members::<Schema>(Some(DeclaredApplicationAftermathContract::runtime_alone(
        DeclaredCorrectionMechanism::RecordedInverse(
            DeclaredRecordedInverse::new(
                "restore-prior",
                DeclaredLoweringCorrespondenceRef::new("test-inverse").unwrap(),
                DeclaredAftermathPostcondition::ExactPriorTruth,
                DeclaredPreImageDemand::new([preimage_locus], 256).unwrap(),
            )
            .unwrap(),
        ),
    )))
}

fn index_for<Schema: ApplicationSchema>() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "typed-test",
        1,
        0,
    ))
    .application_schema(Schema::declaration().unwrap())
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

#[test]
fn covered_preimage_demand_installs_with_operation_decision_reads() {
    let index = index_for::<CoveredAftermathSchema>();
    let schema = index
        .bind_application_schema(CoveredAftermathSchema::declaration().unwrap())
        .unwrap();
    let installed = schema
        .installed_operation(ApplicationOperationRef::<
            CoveredAftermathSchema,
            TestOperation<CoveredAftermathSchema>,
            TestInput,
        >::from_declaration())
        .expect("covered demand must install through operation compile");
    let aftermath = installed
        .contracts()
        .aftermath()
        .expect("aftermath compiles onto the operation");
    assert_eq!(
        aftermath.published_posture(),
        crate::facade::PublishedAftermathPosture::Reversible
    );
    assert!(
        installed
            .contracts()
            .graph_reads()
            .roles()
            .iter()
            .flat_map(|role| role.read_scopes())
            .any(|read| matches!(
                read,
                crate::facade::WorthQueryOperationGraphReadScope::NativeProjection(scope)
                    if scope.projection().mask().paths().iter().any(|path| {
                        path.fields().iter().any(|field| field.as_str() == "PrincipalIdentityField")
                    })
            )),
        "coverage must be retained in the operation's installed typed reads"
    );
}

#[test]
fn uncovered_preimage_demand_denies_operation_installation_by_name() {
    let index = index_for::<UncoveredAftermathSchema>();
    let schema = index
        .bind_application_schema(UncoveredAftermathSchema::declaration().unwrap())
        .unwrap();
    let denial = match schema.installed_operation(ApplicationOperationRef::<
        UncoveredAftermathSchema,
        TestOperation<UncoveredAftermathSchema>,
        TestInput,
    >::from_declaration())
    {
        Ok(_) => panic!("uncovered pre-image demand must deny operation installation"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        crate::facade::WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied
    );
}
