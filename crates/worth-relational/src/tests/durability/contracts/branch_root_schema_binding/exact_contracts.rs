use super::*;

#[test]
fn recovered_exact_roots_interpret_records_with_their_own_schema_contracts() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let v1_registry = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_registry();
    let medium = TestDurableMedium::new();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(v1_registry)
        .build_durable(medium.attachment());
    let old = create_entity_outcome(execution, &runtime, "old-contract");
    let old_entity = changed_entities(&old)[0];
    create_branch_from_main(&runtime, "legacy-schema");

    let v2_registry = AspectSchemaFixture {
        schema_version_id: SchemaVersionId(2),
        entity_aspects: vec![
            entity_field_aspect(aspect_key("display"), field_key("display")),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry();
    runtime.set_schema_registry_for_test(v2_registry.clone());
    let mut transaction = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            batch_create("new-contract"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let new = transaction
        .commit(execution, &runtime)
        .expect("v2 schema transition commits");
    let new_entity = changed_entities(&new)[0];
    let _legacy_before_recovery = update_entity_on_branch(
        execution,
        &runtime,
        old_entity,
        "legacy-v1-before-recovery",
        BranchId("legacy-schema".to_owned()),
    );
    crate::tests::support::checkpoint_durably(execution, &runtime)
        .expect("checkpoint retains both exact root schema carriers");
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();

    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(v2_registry)
        .build();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("fresh owner readmits both schema-qualified roots");

    let legacy = recovered
        .branch_identity(&BranchId("legacy-schema".to_owned()))
        .expect("legacy identity recovers");
    let (_, legacy_basis) = recovered
        .observe_branch(&legacy)
        .expect("legacy root is owner-admitted");
    let legacy_snapshot = recovered
        .snapshots()
        .snapshot_for_observation(&legacy_basis.observation())
        .expect("legacy observation opens its exact root");
    let legacy_read = recovered
        .read_truth()
        .read_observation(execution, &legacy_basis.observation())
        .expect("read request remains live")
        .expect("legacy root reads through its v1 carrier");
    let old_record = legacy_read
        .get_entity(old_entity)
        .expect("v1 entity remains present");
    assert_eq!(
        read_entity_name(old_record),
        Some("legacy-v1-before-recovery".to_owned())
    );
    assert_eq!(
        read_entity_aspect_field(old_record, aspect_key("display"), field_key("display")),
        None
    );

    let main = recovered.main_branch_identity();
    let (_, main_basis) = recovered
        .observe_branch(&main)
        .expect("current root is owner-admitted");
    let main_snapshot = recovered
        .snapshots()
        .snapshot_for_observation(&main_basis.observation())
        .expect("current observation opens its exact root");
    let current_read = recovered
        .read_truth()
        .read_observation(execution, &main_basis.observation())
        .expect("read request remains live")
        .expect("current root reads through its v2 carrier");
    let new_record = current_read
        .get_entity(new_entity)
        .expect("v2 entity remains present");
    assert_eq!(
        read_entity_name(new_record),
        Some("new-contract".to_owned())
    );

    let legacy_projection = recovered
        .read_truth()
        .project_snapshot(&legacy_snapshot)
        .expect("legacy exact projection remains bound");
    let current_projection = recovered
        .read_truth()
        .project_snapshot(&main_snapshot)
        .expect("current exact projection remains bound");
    let name_scope =
        crate::facade::runtime::ProjectionAspectScope::whole_aspects([aspect_key("name")]);
    let display_scope =
        crate::facade::runtime::ProjectionAspectScope::whole_aspects([aspect_key("display")]);
    assert_eq!(
        legacy_projection
            .entity_record_with_projection_scope(execution, old_entity, name_scope.clone(), |_| {
                Some(())
            },)
            .expect("read request remains live"),
        Some(())
    );
    assert_eq!(
        current_projection
            .entity_record_with_projection_scope(
                execution,
                new_entity,
                display_scope.clone(),
                |_| Some(()),
            )
            .expect("read request remains live"),
        Some(())
    );
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        legacy_projection
            .entity_record_with_projection_scope(execution, old_entity, display_scope, |_| Some(()))
            .expect("read request remains live")
    }))
    .is_err());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        current_projection
            .entity_record_with_projection_scope(execution, new_entity, name_scope, |_| Some(()))
            .expect("read request remains live")
    }))
    .is_err());

    let _legacy_after_recovery = update_entity_on_branch(
        execution,
        &recovered,
        old_entity,
        "legacy-v1-after-recovery",
        BranchId("legacy-schema".to_owned()),
    );
    let mut v2_meaning_on_v1_root = crate::tests::support::test_owner_begin_transaction_for_branch(
        &recovered,
        BranchId("legacy-schema".to_owned()),
    );
    v2_meaning_on_v1_root
        .push_batch(
            WorkerIntentBatch::new("v2-meaning-on-v1-root").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: old_entity,
                    fields: single_string_aspect_field_patch(
                        aspect_key("display"),
                        field_key("display"),
                        "not-admitted-by-v1",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let denial = v2_meaning_on_v1_root
        .commit(execution, &recovered)
        .expect_err("a retained v1 root must deny v2-only meaning");
    assert!(matches!(
        denial,
        TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::transactions::data::ConflictClass::RecordAspectPatchDenied {
                    denial: crate::transactions::data::RecordAspectPatchDenial::FieldAuthoringDenied {
                        reason: crate::transactions::data::AspectFieldTargetRejectionReason::UndeclaredAspect,
                        ..
                    },
                    ..
                }
            )
    ));
}
