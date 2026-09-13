use super::*;

use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef,
    DeclaredPreImageDemand, DeclaredPreImageLocus, DeclaredRecordedInverse,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity,
    ApplicationFieldMarkerIdentity, ApplicationFieldPresence,
    ApplicationOperationDecisionReadTarget, ApplicationSchemaMember, DeclaredApplicationFieldValue,
    OperationDeletes, OperationLinks, OperationUnlinks, OperationWrites,
    U64ApplicationValueBinding,
};

struct OtherEntity;
struct OtherEntityAspect;
struct OtherEntityField;
struct OtherAspect;
struct OtherAspectField;
struct OtherField;

impl ApplicationEntityMarkerIdentity<TestSchema> for OtherEntity {
    const IDENTIFIER: &'static str = "OtherEntity";
}

impl ApplicationAspectMarkerIdentity<TestSchema, OtherEntity> for OtherEntityAspect {
    const IDENTIFIER: &'static str = "IdentityAspect";
    const ASPECT_IDENTITY: worth_query_declaration::facade::application_schema::AspectIdentity =
        worth_query_declaration::facade::application_schema::AspectIdentity(0x9161200d);
    const CONTRACT_REVISION:
        worth_query_declaration::facade::application_schema::AspectContractRevision =
        worth_query_declaration::facade::application_schema::AspectContractRevision(1);
}

impl ApplicationFieldMarkerIdentity<TestSchema, OtherEntity, OtherEntityAspect>
    for OtherEntityField
{
    const IDENTIFIER: &'static str = "PrincipalIdentityField";
}
impl DeclaredApplicationFieldValue for OtherEntityField {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}

impl ApplicationAspectMarkerIdentity<TestSchema, TestEntity> for OtherAspect {
    const IDENTIFIER: &'static str = "OtherAspect";
    const ASPECT_IDENTITY: worth_query_declaration::facade::application_schema::AspectIdentity =
        worth_query_declaration::facade::application_schema::AspectIdentity(0x9161200e);
    const CONTRACT_REVISION:
        worth_query_declaration::facade::application_schema::AspectContractRevision =
        worth_query_declaration::facade::application_schema::AspectContractRevision(1);
}

impl ApplicationFieldMarkerIdentity<TestSchema, TestEntity, OtherAspect> for OtherAspectField {
    const IDENTIFIER: &'static str = "PrincipalIdentityField";
}
impl DeclaredApplicationFieldValue for OtherAspectField {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}

impl ApplicationFieldMarkerIdentity<TestSchema, TestEntity, FixtureIdentityAspect<TestSchema>>
    for OtherField
{
    const IDENTIFIER: &'static str = "OtherField";
}
impl DeclaredApplicationFieldValue for OtherField {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}

impl OperationReads<TestOperation<TestSchema>> for FixtureExternalIdentityField<TestSchema> {}
impl OperationReads<TestOperation<TestSchema>> for FixtureEntity<TestSchema> {}
impl OperationReads<TestOperation<TestSchema>> for MappingTarget {}
impl OperationDeletes<TestOperation<TestSchema>> for FixtureEntity<TestSchema> {}
impl OperationWrites<TestOperation<TestSchema>> for FixtureMappingStatusField<TestSchema> {}
impl OperationLinks<TestOperation<TestSchema>> for MappingTarget {}
impl OperationUnlinks<TestOperation<TestSchema>> for MappingTarget {}

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
    assert_eq!(projection.aspect().as_str(), "IdentityAspect");
    assert_eq!(projection.projection().contract().identity().0, 0x9161200c);
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
    assert_eq!(operation.contracts().projection_work_budget(), 32);
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

#[test]
fn installed_contracts_group_exact_reads_and_retain_every_typed_graph_touch() {
    let entity =
        ApplicationEntityRef::<TestSchema, TestEntity>::from_schema_identifier("TestEntity");
    let operation = ApplicationOperationRef::<
        TestSchema,
        TestOperation<TestSchema>,
        TestInput,
    >::from_declaration();
    let relation = ApplicationRelationRef::<
        TestSchema,
        MappingTarget,
        TestEntity,
        TestEntity,
    >::from_schema_identifiers("MappingTarget", "TestEntity", "TestEntity",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let declaration = test_schema_members::<TestSchema>(None)
        .operation_read_field(
            operation,
            ApplicationFieldRef::<
                TestSchema,
                TestEntity,
                FixtureIdentityAspect<TestSchema>,
                FixtureExternalIdentityField<TestSchema>,
                WorthQueryExternalPrincipalIdentity,
                ReadOnly,
                EqualityPredicate,
            >::from_schema_types(),
        )
        .operation_read_entity(operation, entity)
        .operation_read_relation(operation, relation)
        .operation_delete(operation, entity)
        .operation_write(
            operation,
            ApplicationFieldRef::<
                TestSchema,
                TestEntity,
                FixtureIdentityAspect<TestSchema>,
                FixtureMappingStatusField<TestSchema>,
                WorthQueryPrincipalMappingStatus,
                ReadWrite,
                NoEqualityPredicate,
            >::from_schema_types(),
        )
        .operation_link(operation, relation)
        .operation_unlink(operation, relation)
        .build()
        .unwrap();
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
    let declaration = test_schema_members::<TestSchema>(Some(recorded_inverse_at::<
        TestEntity,
        FixtureIdentityAspect<TestSchema>,
        FixturePrincipalIdentityField<TestSchema>,
    >(64)))
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
        coherent_candidate::<OtherEntity, OtherEntityAspect, OtherEntityField>(64),
        coherent_candidate::<TestEntity, OtherAspect, OtherAspectField>(64),
        coherent_candidate::<TestEntity, FixtureIdentityAspect<TestSchema>, OtherField>(64),
        coherent_candidate::<
            TestEntity,
            FixtureIdentityAspect<TestSchema>,
            FixturePrincipalIdentityField<TestSchema>,
        >(65),
    ] {
        assert!(
            !operation.meaning_matches(&changed),
            "the real reinstallation owner must reject one-axis semantic drift"
        );
    }
}

fn installed_index_for(
    declaration: ApplicationSchemaDeclaration<TestSchema>,
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

fn coherent_candidate<Entity, Aspect, Field>(
    maximum_encoded_bytes: usize,
) -> Vec<ApplicationSchemaMember>
where
    Entity: ApplicationEntityMarkerIdentity<TestSchema>,
    Aspect: ApplicationAspectMarkerIdentity<TestSchema, Entity>,
    Field: ApplicationFieldMarkerIdentity<TestSchema, Entity, Aspect, Value = u64>,
{
    let declaration =
        test_schema_members::<TestSchema>(Some(recorded_inverse_at::<Entity, Aspect, Field>(
            maximum_encoded_bytes,
        )))
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
                        entity: Entity::IDENTIFIER.to_owned(),
                        aspect: Aspect::IDENTIFIER.to_owned(),
                        field: Field::IDENTIFIER.to_owned(),
                    },
                }
            }
            member => member,
        })
        .collect()
}

fn recorded_inverse_at<Entity, Aspect, Field>(
    maximum_encoded_bytes: usize,
) -> DeclaredApplicationAftermathContract<TestSchema>
where
    Entity: ApplicationEntityMarkerIdentity<TestSchema>,
    Aspect: ApplicationAspectMarkerIdentity<TestSchema, Entity>,
    Field: ApplicationFieldMarkerIdentity<TestSchema, Entity, Aspect, Value = u64>,
{
    let field = ApplicationFieldRef::<TestSchema, Entity, Aspect, Field, u64>::from_schema_types();
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
