//! Mixed-schema source recovery preserves the original planning root and prefix.
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
fn transition_source_recovers_the_exact_durable_prefix_and_original_planning_root() {
    let policy = serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (source, medium, predecessor, association) = migrated_source(&request);
    let mut receiver = initial_input_source::receiver(&medium);
    let plan = recovery_plan(&request, &receiver, &medium);
    let recovered = receiver
        .durability_recovery()
        .recover_home_program_transition_source(&request, plan, &predecessor, &[association])
        .expect("the original associated migration prefix readmits")
        .consume(|source| source);
    let (outcome, authority, planning) = recovered.into_parts();
    let source_head = durable_publication::current(&request, &source);
    let RecordedRelationalComponentTarget::Canonical(source_head) = source_head.target() else {
        panic!("the source has its actual committed migration")
    };
    assert_eq!(outcome.latest_commit.as_ref(), Some(source_head.receipt()));
    let planning = planning.into_basis();
    durable_publication::assert_recorded_basis(&receiver, &planning, &predecessor);
    let (_, current) = receiver
        .observe_branch(&receiver.main_branch_identity())
        .unwrap();
    assert_ne!(
        planning.descriptor().truth_version(),
        current.descriptor().truth_version()
    );
    assert!(
        authority.admit_basis(planning).is_err(),
        "a historical planning root is not the current execution basis"
    );
    assert_eq!(
        receiver
            .config()
            .schema
            .registry
            .authoritative_schema_basis()
            .unwrap()
            .unwrap()
            .1,
        SchemaVersionId(2)
    );
}

#[test]
fn transition_source_reads_its_planning_predecessor_from_the_selected_admission_image() {
    let policy = serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (source, medium) = initial_input_source::initialized(&request);
    durable_publication::create_entity(&request, &source, "predecessor");
    let predecessor = durable_publication::current(&request, &source);
    let image = initial_input_source::capture(&source, &request);
    medium.file_checkpoint(image.checkpoint().clone());
    let association = publish_migration_step(&request, &source);
    let mut receiver = initial_input_source::receiver(&medium);
    let input = initial_input_source::home_image_input(&medium, &image, &request).unwrap();
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            &request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let recovered = receiver
        .durability_recovery()
        .recover_home_program_transition_source(&request, plan, &predecessor, &[association])
        .expect("the original selected image and its exact migration tail readmit")
        .consume(|source| source);
    let (_, _, planning) = recovered.into_parts();
    let planning = planning.into_basis();
    durable_publication::assert_recorded_basis(&receiver, &planning, &predecessor);
    let before = receiver
        .read_truth()
        .project_observation(&planning.observation())
        .unwrap();
    assert_eq!(
        before
            .bounded_entities_of_kind(&request, worth_relational::facade::identity::KindId(1), 10)
            .unwrap()
            .unwrap()
            .records()
            .len(),
        1
    );
    let (_, current) = receiver
        .observe_branch(&receiver.main_branch_identity())
        .unwrap();
    let after = receiver
        .read_truth()
        .project_observation(&current.observation())
        .unwrap();
    assert_eq!(
        after
            .bounded_entities_of_kind(&request, worth_relational::facade::identity::KindId(1), 10)
            .unwrap()
            .unwrap()
            .records()
            .len(),
        2
    );
}

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

#[test]
fn transition_source_refuses_a_missing_or_foreign_prefix_before_runtime_cutover() {
    let policy = serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (_, medium, predecessor, _) = migrated_source(&request);
    for prefix in [
        vec![],
        vec![RelationalCallerAssociation::from_bytes(b"another-plan-step").unwrap()],
    ] {
        let mut receiver = initial_input_source::receiver(&medium);
        let plan = recovery_plan(&request, &receiver, &medium);
        let failure = receiver
            .durability_recovery()
            .recover_home_program_transition_source(&request, plan, &predecessor, &prefix)
            .err()
            .expect("no supplied description can replace the actual prefix");
        match (prefix.len(), failure) {
            (
                0,
                RelationalHomeProgramTransitionSourceRefusal::PrefixLength {
                    expected: 0,
                    actual: 1,
                },
            ) => {}
            (1, RelationalHomeProgramTransitionSourceRefusal::PrefixAssociation { position }) => {
                assert_eq!(position.0, 2)
            }
            (_, failure) => panic!("the exact prefix evidence refuses: {failure:?}"),
        }
        assert_unrecovered(&receiver);
    }
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
