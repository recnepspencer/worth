mod optional_projection_budget;

use super::*;

use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef,
    DeclaredPreImageDemand, DeclaredPreImageLocus, DeclaredRecordedInverse,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaMember, OperationDeletes,
    OperationLinks, OperationUnlinks,
};

impl OperationReads<TestOperation<TestSchema>> for FixtureEntity {}
impl OperationReads<TestOperation<AllTouchesSchema>> for FixtureEntity {}
impl OperationReads<TestOperation<TestSchema>> for MappingTarget {}
impl OperationReads<TestOperation<AllTouchesSchema>> for MappingTarget {}
impl OperationDeletes<TestOperation<TestSchema>> for FixtureEntity {}
impl OperationDeletes<TestOperation<AllTouchesSchema>> for FixtureEntity {}

impl OperationLinks<TestOperation<TestSchema>> for MappingTarget {}
impl OperationLinks<TestOperation<AllTouchesSchema>> for MappingTarget {}
impl OperationUnlinks<TestOperation<TestSchema>> for MappingTarget {}
impl OperationUnlinks<TestOperation<AllTouchesSchema>> for MappingTarget {}

#[test]
fn installed_application_operation_compiles_existing_authority_contract_families() {
    let index = installed_index();
    let schema = index
        .bind_application_schema(TestSchema::declaration().unwrap())
        .unwrap();
    let operation = schema
        .installed_operation(ApplicationOperationRef::<
            TestSchema,
            TestOperation<TestSchema>,
            TestInput,
        >::from_declaration())
        .unwrap();
    index.validate_application_operation(&operation).unwrap();

    let obligations = operation.graph_obligations();
    assert_eq!(obligations.rows().len(), 5);
    assert!(obligations.rows().iter().any(|row| {
        row.kind() == crate::facade::WorthQueryInstalledGraphObligationKind::InvariantExecution
            && row.invariant_requirement().is_some()
    }));
    assert_eq!(
        obligations
            .installation_evidence()
            .canonical_work()
            .digest_text_materializations(),
        0
    );

    let authorization = &operation.contracts().ability_requirements()[0];
    assert_ne!(authorization.identity().bytes(), &[0; 32]);
    assert!(
        authorization.canonical_work().digest_derivations() >= 2,
        "the installed policy identity and its path identity must both be phase-accounted"
    );
    assert_eq!(authorization.ability(), "TestAbility");
    assert!(matches!(
        operation.contracts().graph_reads(),
        crate::facade::WorthQueryOperationGraphReadContract::Declared { roles }
            if roles.len() == 1 && roles[0].role() == "primary"
    ));
    let read = &operation.contracts().graph_reads().roles()[0].read_scopes()[0];
    let crate::facade::WorthQueryOperationGraphReadScope::NativeProjection(projection) = read
    else {
        panic!("the declared field read must compile to a native projection")
    };
    assert_eq!(projection.schema(), operation.binding_identity());
    assert_eq!(projection.entity().semantic_key(), "TestEntity");
    assert_eq!(
        projection.aspect().as_str(),
        worth_query_declaration::facade::application_schema::fragment::RecordSelection::aspect(
            FixturePrincipalIdentityField::at("PrincipalIdentityField")
        )
    );
    let declaration = TestSchema::declaration().unwrap();
    let expected = declaration
        .erased()
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::Aspect {
                entity,
                aspect,
                identity,
                ..
            } if entity == "TestEntity" && aspect == projection.aspect().as_str() => {
                Some(*identity)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(projection.projection().contract().identity(), expected);
    assert!(!projection.projection().mask().is_whole_aspect());
    assert_eq!(
        projection.projection().mask().paths(),
        &[CanonicalFieldPath::single(
            FieldKey::new("PrincipalIdentityField").unwrap()
        )]
    );
    assert!(matches!(
        operation.contracts().touches(),
        crate::facade::WorthQueryOperationTouchContract::Declared { scopes, .. }
            if matches!(scopes.as_slice(), [crate::facade::WorthQueryOperationTouchScope::CreateEntity(scope)] if scope.entity() == "TestEntity")
    ));
    assert_eq!(operation.contracts().read_touch_overlap().reads().len(), 1);
    let [precondition] = operation.contracts().mutation_preconditions() else {
        panic!("the exact declared mutation precondition must be installed");
    };
    assert_eq!(
        precondition.target().family(),
        worth_query_declaration::facade::application_schema::ApplicationMutationPreconditionFamily::ExpectedFact
    );
    assert_eq!(precondition.target().entity(), "TestEntity");
    assert_eq!(precondition.target().field_name(), "PrincipalIdentityField");
    assert_eq!(operation.contracts().projection_work_budget(), Some(32));
    assert!(matches!(
        operation.contracts().effects(),
        crate::facade::WorthQueryOperationEffectContract::Declared { effect_families }
            if effect_families == &[crate::facade::WorthQueryOperationEffectFamily::Mutation]
    ));
    assert_eq!(
        operation
            .contracts()
            .execution_strategy()
            .expect("compiled application operation must have one execution strategy")
            .name()
            .as_str(),
        "primary-application-atomic"
    );
}

struct AllTouchesSchema;
impl ApplicationSchema for AllTouchesSchema {
    const OWNER: &'static str = TestSchema::OWNER;
    const NAME: &'static str = "AllTouchesSchema";
    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let entity = worth_query_declaration::facade::application_schema::fragment::entity(
            InstallationFields,
            FixtureEntity,
        );
        let operation = ApplicationOperationRef::<
            AllTouchesSchema,
            TestOperation<AllTouchesSchema>,
            TestInput,
        >::from_declaration();
        let relation = ApplicationRelationRef::<
        AllTouchesSchema,
        MappingTarget,
        TestEntity,
        TestEntity,
    >::from_schema_identifiers("MappingTarget", "TestEntity", "TestEntity",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
        test_schema_members::<AllTouchesSchema>(None)
            .operation_read_field(
                operation,
                worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
                    InstallationFields,
                    TestEntity,
                    FixtureIdentityAspect,
                    WorthQueryExternalPrincipalIdentity,
                >::at("ExternalIdentityField"),
            )
            .operation_read_entity(operation, entity)
            .operation_read_relation(operation, relation)
            .operation_delete(operation, entity)
            .operation_write(
                operation,
                worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
                    InstallationFields,
                    TestEntity,
                    FixtureIdentityAspect,
                    WorthQueryPrincipalMappingStatus,
                >::at("MappingStatusField"),
            )
            .operation_link(operation, relation)
            .operation_unlink(operation, relation)
            .build()
    }
}

#[test]
fn installed_contracts_group_exact_reads_and_retain_every_typed_graph_touch() {
    let declaration = AllTouchesSchema::declaration().unwrap();
    let operation = ApplicationOperationRef::<
        AllTouchesSchema,
        TestOperation<AllTouchesSchema>,
        TestInput,
    >::from_declaration();
    let index = installed_index_for(declaration.clone());
    let schema = index.bind_application_schema(declaration).unwrap();
    let installed = schema.installed_operation(operation).unwrap();

    let scopes = installed.contracts().graph_reads().roles()[0].read_scopes();
    assert_eq!(
        scopes.len(),
        3,
        "two field reads share one projection scope"
    );
    let projection = scopes
        .iter()
        .find_map(|scope| match scope {
            crate::facade::WorthQueryOperationGraphReadScope::NativeProjection(scope) => {
                Some(scope)
            }
            crate::facade::WorthQueryOperationGraphReadScope::Entity(_)
            | crate::facade::WorthQueryOperationGraphReadScope::Components(_)
            | crate::facade::WorthQueryOperationGraphReadScope::DerivedSource(_)
            | crate::facade::WorthQueryOperationGraphReadScope::Relation(_) => None,
        })
        .unwrap();
    assert!(!projection.projection().mask().is_whole_aspect());
    assert_eq!(projection.projection().mask().paths().len(), 2);

    let touches = installed.contracts().touches().scopes();
    assert_eq!(touches.len(), 5);
    assert!(touches.iter().any(|scope| matches!(
        scope,
        crate::facade::WorthQueryOperationTouchScope::CreateEntity(_)
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        crate::facade::WorthQueryOperationTouchScope::DeleteEntity(_)
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        crate::facade::WorthQueryOperationTouchScope::WriteField(_)
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        crate::facade::WorthQueryOperationTouchScope::LinkRelation(_)
    )));
    assert!(touches.iter().any(|scope| matches!(
        scope,
        crate::facade::WorthQueryOperationTouchScope::UnlinkRelation(_)
    )));
}

#[test]
fn reinstallation_rejects_each_coherent_preimage_axis_and_bound_drift() {
    let declaration = test_schema_members::<TestSchema>(Some(recorded_inverse_at(64)))
        .build()
        .unwrap();
    let index = installed_index_for(declaration.clone());
    let schema = index.bind_application_schema(declaration.clone()).unwrap();
    let operation = schema
        .installed_operation(ApplicationOperationRef::<
            TestSchema,
            TestOperation<TestSchema>,
            TestInput,
        >::from_declaration())
        .unwrap();

    assert!(operation.meaning_matches(declaration.erased().members()));
    for changed in [
        coherent_candidate("entity", 64),
        coherent_candidate("aspect", 64),
        coherent_candidate("field", 64),
        coherent_candidate("bound", 65),
    ] {
        assert!(
            !operation.meaning_matches(&changed),
            "the real reinstallation owner must reject one-axis semantic drift"
        );
    }
}

fn installed_index_for<Schema: ApplicationSchema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
) -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "typed-test",
        1,
        0,
    ))
    .application_schema(declaration)
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

fn coherent_candidate(axis: &str, maximum_encoded_bytes: usize) -> Vec<ApplicationSchemaMember> {
    use worth_query_declaration::facade::application_aftermath::{
        PortableApplicationAftermathContract, PortableCorrectionMechanism, PortablePreImageDemand,
        PortablePreImageLocus, PortableRecordedInverse,
    };

    let field = FixturePrincipalIdentityField::at("PrincipalIdentityField");
    let entity = if axis == "entity" {
        "OtherEntity"
    } else {
        field.entity()
    };
    let aspect = if axis == "aspect" {
        "OtherAspect"
    } else {
        field.aspect()
    };
    let name = if axis == "field" {
        "OtherField"
    } else {
        field.field()
    };
    let declaration = test_schema_members::<TestSchema>(Some(recorded_inverse_at(64)))
        .build()
        .unwrap();
    declaration
        .erased()
        .members()
        .iter()
        .cloned()
        .map(|member| match member {
            ApplicationSchemaMember::OperationDecisionRead { operation, .. }
                if operation == "TestOperation" =>
            {
                ApplicationSchemaMember::OperationDecisionRead {
                    operation,
                    target: ApplicationOperationDecisionReadTarget::Field {
                        entity: entity.into(),
                        aspect: aspect.into(),
                        field: name.into(),
                    },
                }
            }
            ApplicationSchemaMember::OperationAftermath {
                operation,
                contract,
            } if operation == "TestOperation" => {
                let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) =
                    contract.mechanism()
                else {
                    panic!("recorded inverse fixture")
                };
                let changed = PortableRecordedInverse::from_untrusted_fields(
                    inverse.inverse_operation_slot().into(),
                    inverse.lowering_correspondence().clone(),
                    inverse.postcondition().clone(),
                    PortablePreImageDemand::from_untrusted_fields(
                        vec![PortablePreImageLocus::from_untrusted_fields(
                            entity.into(),
                            aspect.into(),
                            name.into(),
                        )],
                        maximum_encoded_bytes,
                    ),
                );
                ApplicationSchemaMember::OperationAftermath {
                    operation,
                    contract: PortableApplicationAftermathContract::from_untrusted_fields(
                        contract.authority(),
                        Some(PortableCorrectionMechanism::RecordedInverse(changed)),
                        contract.reconciliation().cloned(),
                    ),
                }
            }
            member => member,
        })
        .collect()
}

fn recorded_inverse_at(
    maximum_encoded_bytes: usize,
) -> DeclaredApplicationAftermathContract<TestSchema> {
    let field = FixturePrincipalIdentityField::at("PrincipalIdentityField");
    let inverse = DeclaredRecordedInverse::new(
        "restore-test-operation",
        DeclaredLoweringCorrespondenceRef::new("test-operation-inverse").unwrap(),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(field)],
            maximum_encoded_bytes,
        )
        .unwrap(),
    )
    .unwrap();
    DeclaredApplicationAftermathContract::runtime_alone(
        DeclaredCorrectionMechanism::RecordedInverse(inverse),
    )
}
