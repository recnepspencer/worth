use super::*;

#[test]
fn branch_roots_with_one_schema_deduplicate_the_durable_carrier() {
    let runtime = persisted_runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "schema-carrier-deduplication");
    let entity = changed_entities(&created)[0];
    create_branch_from_main(&runtime, "feature");
    update_entity_on_branch(
        &runtime,
        entity,
        "schema-carrier-feature-root",
        BranchId("feature".to_owned()),
    );

    let checkpoint = runtime
        .durability_authority()
        .checkpoint()
        .expect("production checkpoint succeeds");

    assert!(checkpoint.branch_roots.len() >= 2);
    assert_eq!(checkpoint.branch_root_schema_images.len(), 1);
}

#[test]
fn recovery_rejects_a_missing_branch_root_schema_carrier() {
    let mut plan = checkpoint_plan_with_one_root("missing-schema-carrier");
    plan.checkpoint
        .as_mut()
        .expect("checkpoint exists")
        .branch_root_schema_images
        .clear();

    assert_corrupt_schema_recovery(plan, "missing schema carrier");
}

#[test]
fn recovery_rejects_a_tampered_branch_root_schema_carrier() {
    let mut plan = checkpoint_plan_with_one_root("tampered-schema-carrier");
    plan.checkpoint
        .as_mut()
        .and_then(|checkpoint| checkpoint.branch_root_schema_images.first_mut())
        .expect("checkpoint carries a schema carrier")
        .corrupt_schema_root_for_test();

    assert_corrupt_schema_recovery(plan, "schema carrier digest mismatch");
}

#[test]
fn recovery_rejects_a_schema_carrier_swapped_between_exact_roots() {
    let mut runtime =
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_outcome(&runtime, "schema-v1");
    create_branch_from_main(&runtime, "legacy");
    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
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
            batch_create("schema-v2"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("schema transition commits");
    runtime
        .durability_authority()
        .checkpoint()
        .expect("production checkpoint succeeds");
    let mut plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint exists");
    assert_eq!(checkpoint.branch_root_schema_images.len(), 2);
    let replacement = checkpoint.branch_roots[1].schema_carrier_digest;
    let root = &mut checkpoint.branch_roots[0];
    assert_ne!(root.schema_carrier_digest, replacement);
    root.schema_carrier_digest = replacement;
    root.root_image_digest = crate::durability::data::branch_root_image_digest(
        root.format_version,
        root.commit_id,
        root.partition_image_digest,
        root.schema_carrier_digest,
    );

    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(
            AspectSchemaFixture {
                schema_version_id: SchemaVersionId(2),
                ..AspectSchemaFixture::with_default_declared_aspects(
                    CascadeDeletePolicy::CascadeDeleteRelations,
                )
            }
            .build_registry(),
        )
        .build();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("swapped root schema carrier cannot be readmitted");
    assert_eq!(
        error.class,
        RecoveryFailureClass::CorruptCheckpoint,
        "unexpected denial: {error:?}"
    );
    assert!(
        error.detail.contains("schema carrier linkage mismatch"),
        "unexpected denial: {error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn recovered_exact_roots_interpret_records_with_their_own_schema_contracts() {
    let v1_registry = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_registry();
    let store_layout = DurableStoreLayout {
        root_path: unique_test_store_path("worth-relational-root-schema-interpretation"),
        segment_commit_capacity: 2,
    };
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(v1_registry)
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout)
        .build();
    let old = create_entity_outcome(&runtime, "old-contract");
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
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("v2 schema transition commits");
    let new_entity = changed_entities(&new)[0];
    let _legacy_before_recovery = update_entity_on_branch(
        &runtime,
        old_entity,
        "legacy-v1-before-recovery",
        BranchId("legacy-schema".to_owned()),
    );
    runtime
        .durability_authority()
        .checkpoint()
        .expect("checkpoint retains both exact root schema carriers");
    let plan = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);

    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(v2_registry)
        .build();
    recovered
        .durability_recovery()
        .recover(plan)
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
        .read_observation(&legacy_basis.observation())
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
        .read_observation(&main_basis.observation())
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
        legacy_projection.entity_record_with_projection_scope(
            old_entity,
            name_scope.clone(),
            |_| Some(()),
        ),
        Some(())
    );
    assert_eq!(
        current_projection.entity_record_with_projection_scope(
            new_entity,
            display_scope.clone(),
            |_| Some(()),
        ),
        Some(())
    );
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        legacy_projection
            .entity_record_with_projection_scope(old_entity, display_scope, |_| Some(()))
    }))
    .is_err());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        current_projection.entity_record_with_projection_scope(new_entity, name_scope, |_| Some(()))
    }))
    .is_err());

    let _legacy_after_recovery = update_entity_on_branch(
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
        .commit(
            &recovered,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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

fn checkpoint_plan_with_one_root(label: &str) -> RecoveryPlan {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, label);
    runtime
        .durability_authority()
        .checkpoint()
        .expect("production checkpoint succeeds");
    runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification)
}

fn assert_corrupt_schema_recovery(plan: RecoveryPlan, expected_detail: &str) {
    assert_schema_recovery_denial(
        plan,
        RecoveryFailureClass::CorruptCheckpoint,
        expected_detail,
    );
}

fn assert_schema_recovery_denial(
    plan: RecoveryPlan,
    expected_class: RecoveryFailureClass,
    expected_detail: &str,
) {
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(plan)
        .expect_err("corrupt root schema carrier cannot be readmitted");

    assert_eq!(error.class, expected_class, "unexpected denial: {error:?}");
    assert!(
        error.detail.contains(expected_detail),
        "unexpected denial: {error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}
