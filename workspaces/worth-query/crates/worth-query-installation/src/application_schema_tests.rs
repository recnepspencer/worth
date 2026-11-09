use worth_query_declaration::facade::application_aftermath::DeclaredApplicationAftermathContract;
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationAuthorizationPathBuilder, ApplicationEffectRef,
    ApplicationEntityRef, ApplicationOperationMarkerIdentity, ApplicationOperationRef,
    ApplicationPolicyRef, ApplicationPrincipalBindingRef, ApplicationPrincipalBindingRequirements,
    ApplicationPrincipalIdentityRequirement, ApplicationPrincipalMappingIdentityRequirement,
    ApplicationPrincipalMappingStatusRequirement, ApplicationPrincipalTargetRequirement,
    ApplicationRelationRef, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, OperationCreates, OperationReads,
    OperationRequiresAbility,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};

use crate::facade::{
    WorthQueryAbilityInstallationDenialKind, WorthQueryInstallationAdmissionProfile,
    WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity,
    WorthQueryInstalledApplicationSchemaDenialKind, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDefinition, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

mod aftermath_coverage;
mod conditional_application_operation;
pub(crate) use conditional_application_operation::complete_typed_package_fixture;
mod declaration_provenance;
mod derived_relation;
mod effect_fixture;
mod field_references;
mod installed_views;
mod mutation_binding;
mod native_contract_catalog;
mod operation_contracts;
mod package_schema_identity;
mod portable_record_assertions;
mod principal_binding;
mod projected_member_domain;
mod read_only_operations;
mod schema_affinity;
mod value_bindings;

#[test]
fn admitted_operation_budget_counts_every_declared_family() {
    let schema = installed_index()
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let families = crate::declaration_budget_tests::assert_complete_source_budget(&schema);
    for family in ["field", "ability-policy", "operation-program"] {
        assert!(
            families.contains(family),
            "the real fixture must exercise {family}"
        );
    }
}

#[test]
fn admitted_schema_budget_covers_every_declaration_family() {
    let declaration = crate::declaration_budget_tests::all_families_declaration();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "worth.budget-fixture",
        1,
        0,
    ))
    .application_schema_erased(declaration.clone())
    .validate()
    .expect("real portable package admission");
    assert_eq!(package.application_schemas(), &[declaration]);
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .expect("real installation admission");
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("the complete schema must install through the real index owner");
    assert_eq!(
        index.assert_admitted_declaration_budgets(),
        crate::declaration_budget_tests::required_families(),
        "every compiler-enumerated declaration family must reach this real schema"
    );
}

use effect_fixture::{TestEffect, TestPayload, TestPayloadBinding};
use field_references::*;
use principal_binding::test_principal_binding;

struct TestSchema;
struct DriftedSchema;
struct AddedEntity;

impl worth_query_declaration::facade::application_schema::fragment::Entity for AddedEntity {
    const NAME: &'static str = "AddedEntity";
    type Aspects = ();
    type Relations = ();
}
struct DriftedFields;
impl worth_query_declaration::facade::application_schema::fragment::Fragment for DriftedFields {
    const NAME: &'static str = "drifted_fields";
    type Bindings = worth_query_declaration::facade::application_schema::fragment::NoBindings;
    type Members = (InstallationFields, AddedEntity);
}
struct TestAbility;
struct TestOperation<Schema>(std::marker::PhantomData<Schema>);
#[derive(serde::Serialize)]
struct TestInput;
struct MappingTarget;
struct PrincipalBinding;
struct TestPolicy;

worth_query_declaration::worth_query_portable_type!(
    TestInput => "worth.query.installation-test.operation-input"
);
worth_query_declaration::worth_query_structured_value_binding!(
    TestInputBinding for TestInput { identity: "worth.query.installation-test.operation-input" }
);
impl<Schema> ApplicationOperationMarkerIdentity<Schema> for TestOperation<Schema> {
    type InputBinding = TestInputBinding;
    const IDENTIFIER: &'static str = "TestOperation";
}

impl<Schema> OperationRequiresAbility<TestOperation<Schema>> for TestAbility {}

impl ApplicationSchema for TestSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "TestSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        test_schema_members::<Self>(None).build()
    }
}

impl ApplicationSchema for DriftedSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "TestSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        test_schema_members_for::<Self, TestOperation<Self>>(
            None,
            ApplicationSchemaDeclarationBuilder::from_fragment(DriftedFields)?,
        )
        .build()
    }
}

fn test_schema_members<Schema>(
    aftermath: Option<DeclaredApplicationAftermathContract<Schema>>,
) -> ApplicationSchemaDeclarationBuilder<Schema>
where
    Schema: ApplicationSchema,
{
    test_schema_members_for::<Schema, TestOperation<Schema>>(
        aftermath,
        ApplicationSchemaDeclarationBuilder::from_fragment(InstallationFields).unwrap(),
    )
}

fn test_schema_members_for<Schema, Operation>(
    aftermath: Option<DeclaredApplicationAftermathContract<Schema>>,
    storage: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema>
where
    Schema: ApplicationSchema,
    Operation:
        ApplicationOperationMarkerIdentity<Schema, InputBinding = TestInputBinding> + 'static,
    TestAbility: OperationRequiresAbility<Operation>,
    FixtureEntity: OperationCreates<Operation>,
{
    let entity =
        ApplicationEntityRef::<Schema, FixtureEntity>::from_schema_identifier("TestEntity");
    let ability =
        ApplicationAbilityRef::<Schema, TestAbility, FixtureEntity>::from_schema_identifiers(
            "TestAbility",
            "TestEntity",
        );
    let operation = ApplicationOperationRef::<Schema, Operation, TestInput>::from_declaration();
    let operation_definition = match aftermath {
        Some(contract) => operation
            .definition()
            .no_external_effect()
            .aftermath(contract)
            .finish(),
        None => operation
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish(),
    };
    storage
        .principal_binding(test_principal_binding::<Schema>())
        .policy(ApplicationPolicyRef::<Schema, TestPolicy>::from_schema_identifier("TestPolicy"))
        .effect(ApplicationEffectRef::<
            Schema,
            TestEffect<Schema>,
            TestPayload,
        >::from_declaration())
        .ability(ability)
        .ability_policy(
            ability,
            ApplicationPolicyRef::<Schema, TestPolicy>::from_schema_identifier("TestPolicy"),
            [ApplicationAuthorizationPathBuilder::from_principal(entity).allow(entity)],
        )
        .operation(operation_definition)
        .operation_projection_work_budget(operation, 32)
        .operation_requires_ability(operation, ability)
        .operation_read_field(
            operation,
            worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
                InstallationFields,
                FixtureEntity,
                FixtureIdentityAspect,
                u64,
            >::at("PrincipalIdentityField"),
        )
        .operation_expected_fact(
            operation,
            worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
                InstallationFields,
                FixtureEntity,
                FixtureIdentityAspect,
                u64,
            >::at("PrincipalIdentityField"),
        )
        .operation_create(operation, entity)
}

#[test]
fn installed_ability_is_runtime_generation_and_meaning_affine() {
    let index = installed_index();
    let schema = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let ability = schema
        .ability(
            ApplicationAbilityRef::<TestSchema, TestAbility, TestEntity>::from_schema_identifiers(
                "TestAbility",
                "TestEntity",
            ),
        )
        .unwrap();
    index.validate_ability(&ability).unwrap();

    let denial = installed_index().validate_ability(&ability).unwrap_err();
    assert_eq!(
        denial.kind(),
        crate::facade::WorthQueryAbilityInstallationDenialKind::ForeignRuntime
    );
    let denial = index
        .successor_generation()
        .validate_ability(&ability)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        crate::facade::WorthQueryAbilityInstallationDenialKind::StaleGeneration
    );
}

#[test]
fn fabricated_ability_identity_cannot_bind_installed_authority() {
    let index = installed_index();
    let schema = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();

    let unknown = schema
        .ability(
            ApplicationAbilityRef::<TestSchema, TestAbility, TestEntity>::from_schema_identifiers(
                "FabricatedAbility",
                "TestEntity",
            ),
        )
        .expect_err("fabricated ability name must not bind installed authority");
    assert_eq!(
        unknown.kind(),
        WorthQueryAbilityInstallationDenialKind::AbilityNotInstalled
    );

    let changed_scope = schema
        .ability(
            ApplicationAbilityRef::<TestSchema, TestAbility, TestEntity>::from_schema_identifiers(
                "TestAbility",
                "FabricatedScope",
            ),
        )
        .expect_err("fabricated ability scope must not bind installed authority");
    assert_eq!(
        changed_scope.kind(),
        WorthQueryAbilityInstallationDenialKind::AbilityMeaningChanged
    );
}

fn installed_index() -> WorthQueryInstalledPackageIndex {
    installed_index_with(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        false,
        "support",
    )
}

fn installed_index_with(
    runtime: WorthQueryInstallationRuntimeIdentity,
    package_drift: bool,
    support: &str,
) -> WorthQueryInstalledPackageIndex {
    let mut package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "typed-test",
        1,
        0,
    ))
    .application_schema(TestSchema::declaration().unwrap());
    if package_drift {
        package = package.definition(WorthQueryPortableDefinition::declaration_family(
            "extra",
            "package-drift",
        ));
    }
    let package = package.validate().unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new(support, "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        runtime,
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
}

/// Reuse the actual admitted operation/derived declarations in the budget court.
pub(crate) fn budget_declarations(
) -> Vec<worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration> {
    vec![
        mutation_binding::MutationSchema::declaration()
            .unwrap()
            .erased()
            .clone(),
        aftermath_coverage::CoveredAftermathSchema::declaration()
            .unwrap()
            .erased()
            .clone(),
        derived_relation::DerivedSchema::<true>::declaration()
            .unwrap()
            .erased()
            .clone(),
    ]
}
