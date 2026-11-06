//! Ordinary recovery keeps strict declared-schema admission after migration.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{branch::*, durability::*, mvcc::*, runtime::*, schema::*};
use worth_testkit_relational::{
    aspect_storage::{publications::release_test_commit_snapshot, AspectSchemaFixture},
    durable_medium::TestDurableMedium,
    durable_publication, initial_input_source,
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn ordinary_recovery_still_refuses_a_declared_schema_that_differs_from_the_durable_head() {
    let policy = serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (_, medium, _, _) = migrated_source(&request);
    let mut receiver = initial_input_source::receiver(&medium);
    let plan = recovery_plan(&request, &receiver, &medium);
    let failure = receiver
        .durability_recovery()
        .recover_with_authority(&request, plan)
        .unwrap_err();
    let RelationalRecoveryStop::Durability(failure) = failure else {
        panic!("exact schema admission remains the refusing boundary: {failure:?}")
    };
    assert_eq!(failure.class, RecoveryFailureClass::DeclaredSchemaMismatch);
    assert_unrecovered(&receiver);
}

fn assert_unrecovered(runtime: &RelationalRuntime) {
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(matches!(
        current.reference().target(),
        worth_foundational::FoundationalBranchTarget::Empty
    ));
}

fn recovery_plan(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &Arc<TestDurableMedium>,
) -> RecoveryPlan {
    runtime
        .durability()
        .recovery_plan_from_source(
            request,
            initial_input_source::journal_home_input(medium, request).unwrap(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
}

fn migrated_source(
    request: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    Arc<TestDurableMedium>,
    RecordedRelationalComponentReference,
    RelationalCallerAssociation,
) {
    let (runtime, medium) = initial_input_source::initialized(request);
    durable_publication::create_entity(request, &runtime, "predecessor");
    let predecessor = durable_publication::current(request, &runtime);
    let association = publish_migration_step(request, &runtime);
    (runtime, medium, predecessor, association)
}

fn publish_migration_step(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> RelationalCallerAssociation {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let target = target_registry();
    let transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".into()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".into()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Schema,
                SchemaId("test".into()),
                SchemaVersionId(2),
                None,
                "published-basis",
            ),
            vec![SchemaStratum::PublicationContract],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail: "an explicit successor schema basis".into(),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    };
    let mut transaction = runtime
        .begin_branch_schema_transition(&basis, transition, None, target)
        .unwrap();
    transaction
        .push_batch(
            durable_publication::batch_create("durable-step"),
            request.allocation_policy(),
        )
        .unwrap();
    let association =
        RelationalCallerAssociation::from_bytes(b"original-home-transition-step").unwrap();
    let proposal = runtime
        .validate_branch_transaction(request, transaction)
        .unwrap()
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(request, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(request, candidate)
    else {
        panic!("the actual canonical schema migration must commit")
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    association
}

fn target_registry() -> RelationalSchemaRegistry {
    let fixture = AspectSchemaFixture {
        schema_version_id: SchemaVersionId(2),
        ..AspectSchemaFixture::with_default_declared_aspects(
            worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
        )
    };
    let mut extra = initial_input_source::additions()
        .relation_registration(worth_relational::facade::identity::KindId(901))
        .unwrap()
        .clone();
    extra.schema_version_id = SchemaVersionId(2);
    fixture
        .build_registry()
        .register_relation_kind(extra)
        .unwrap()
}
