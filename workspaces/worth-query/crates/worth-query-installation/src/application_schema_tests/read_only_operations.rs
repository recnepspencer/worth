use super::*;

struct ReadTestSchema;
struct ReadOnlyOperation;
struct ReadOnlyInput;
struct EmptyOperation;
struct EmptyInput;

worth_query_declaration::worth_query_portable_type!(
    ReadOnlyInput => "worth.query.installation-test.read-only-input"
);
worth_query_declaration::worth_query_portable_type!(
    EmptyInput => "worth.query.installation-test.empty-input"
);
worth_query_declaration::worth_query_structured_value_binding!(
    ReadOnlyInputBinding for ReadOnlyInput { identity: "worth.query.installation-test.read-only-input" }
);
worth_query_declaration::worth_query_structured_value_binding!(
    EmptyInputBinding for EmptyInput { identity: "worth.query.installation-test.empty-input" }
);

impl ApplicationOperationMarkerIdentity<ReadTestSchema> for ReadOnlyOperation {
    type InputBinding = ReadOnlyInputBinding;
    const IDENTIFIER: &'static str = "ReadOnlyOperation";
}

impl ApplicationOperationMarkerIdentity<ReadTestSchema> for EmptyOperation {
    type InputBinding = EmptyInputBinding;
    const IDENTIFIER: &'static str = "EmptyOperation";
}

impl<Schema> OperationReads<ReadOnlyOperation> for FixturePrincipalIdentityField<Schema> {}
impl OperationRequiresAbility<ReadOnlyOperation> for TestAbility {}
impl OperationRequiresAbility<EmptyOperation> for TestAbility {}

impl ApplicationSchema for ReadTestSchema {
    const OWNER: &'static str = "typed-test";
    const NAME: &'static str = "ReadTestSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let read =
            ApplicationOperationRef::<Self, ReadOnlyOperation, ReadOnlyInput>::from_declaration();
        let empty = ApplicationOperationRef::<Self, EmptyOperation, EmptyInput>::from_declaration();
        let ability =
            ApplicationAbilityRef::<Self, TestAbility, FixtureEntity<Self>>::from_schema_identifiers(
                "TestAbility",
                "TestEntity",
            );
        test_schema_members::<Self>(None)
            .operation(
                read.definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_decision_fact_budget(read, 1)
            .operation_projection_work_budget(read, 16)
            .operation_requires_ability(read, ability)
            .operation_read_field(
                read,
                ApplicationFieldRef::<
                    Self,
                    FixtureEntity<Self>,
                    FixtureIdentityAspect<Self>,
                    FixturePrincipalIdentityField<Self>,
                    u64,
                    ReadOnly,
                    EqualityPredicate,
                >::from_schema_types(),
            )
            .operation(
                empty
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_decision_fact_budget(empty, 1)
            .operation_projection_work_budget(empty, 16)
            .operation_requires_ability(empty, ability)
            .build()
    }
}

#[test]
fn read_only_operation_installs_without_an_effect_program() {
    let index = installed_read_index();
    let schema = index
        .bind_application_schema(ReadTestSchema::declaration().unwrap())
        .unwrap();
    let installed = schema
        .installed_operation(ApplicationOperationRef::<
            ReadTestSchema,
            ReadOnlyOperation,
            ReadOnlyInput,
        >::from_declaration())
        .unwrap();

    assert!(installed.contracts().touches().scopes().is_empty());
    assert_eq!(
        installed
            .contracts()
            .graph_reads()
            .roles()
            .iter()
            .map(|role| role.read_scopes().len())
            .sum::<usize>(),
        1
    );
    assert!(matches!(
        installed.contracts().effects(),
        crate::facade::WorthQueryOperationEffectContract::NotRequired
    ));
    assert!(matches!(
        installed.contracts().touches(),
        crate::facade::WorthQueryOperationTouchContract::NotRequired
    ));
    assert!(matches!(
        installed.contracts().invariants(),
        crate::facade::WorthQueryOperationInvariantContract::NotRequired
    ));
    assert!(matches!(
        installed.contracts().invariant_execution(),
        crate::facade::WorthQueryInvariantExecutionContract::NotRequired
    ));
}

#[test]
fn operation_without_reads_or_effects_remains_uninstallable() {
    let index = installed_read_index();
    let schema = index
        .bind_application_schema(ReadTestSchema::declaration().unwrap())
        .unwrap();
    let denial = match schema.installed_operation(ApplicationOperationRef::<
        ReadTestSchema,
        EmptyOperation,
        EmptyInput,
    >::from_declaration())
    {
        Ok(_) => panic!("an operation without reads or effects must not install"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.kind(),
        crate::facade::WorthQueryApplicationOperationInstallationDenialKind::MissingProgram
    );
}

fn installed_read_index() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "typed-test",
        1,
        0,
    ))
    .application_schema(ReadTestSchema::declaration().unwrap())
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
