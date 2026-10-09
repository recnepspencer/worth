use super::*;

#[test]
fn durability_contract_recovery_rebuilds_branch_head_root_obligations() {
    let runtime = persisted_runtime_with_test_schema();
    let source = create_entity_outcome(&runtime, "source");
    let source_entity = changed_entities(&source)[0];
    let target = create_entity_outcome(&runtime, "target");
    let target_entity = changed_entities(&target)[0];
    let _relation = create_relation_outcome(&runtime, source_entity, target_entity, "r1");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _deleted = delete_entity(&runtime, source_entity);
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();

    let retention = recovered.retention().inspect_plan();
    assert_eq!(retention.active_snapshot_count, 0);
    assert_eq!(retention.branch_pinned_entities, 0);
    assert_eq!(retention.branch_pinned_relations, 0);
    let feature = recovered
        .branch_identity(&BranchId("feature".to_owned()))
        .unwrap();
    let (_, feature_basis) = recovered.observe_branch(&feature).unwrap();
    let feature_truth = recovered
        .read_truth()
        .read_observation(&feature_basis.observation())
        .unwrap();
    assert!(feature_truth.get_entity(source_entity).is_some());
    assert!(feature_truth
        .relations()
        .iter()
        .any(|relation| { relation.source == source_entity && relation.target == target_entity }));
}

#[test]
fn durability_contract_recovery_preserves_inspection_truth_bundle() {
    let runtime = persisted_runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "source");
    let entity = changed_entities(&created)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _main_update = update_entity(&runtime, entity, "main");
    let _feature_update = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
            &runtime,
            BranchId("feature".to_string()),
        );
        txn.push_batch(
            WorkerIntentBatch::new("feature-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "feature",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    };
    runtime.durability_authority().checkpoint().unwrap();
    let expected = capture_inspection_truth_bundle(
        &runtime,
        &BranchId("feature".to_string()),
        entity,
        created.version_id,
    );

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    let actual = capture_inspection_truth_bundle(
        &recovered,
        &BranchId("feature".to_string()),
        entity,
        created.version_id,
    );

    assert_eq!(expected, actual);
}

#[test]
fn durability_contract_branch_heads_do_not_mutate_legacy_record_pins() {
    let runtime = persisted_runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "source");
    let entity = changed_entities(&created)[0];
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(RecordRef::Entity(entity))
            .expect("entity retention after create")
            .pins
            .branch_pins,
        0
    );

    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(RecordRef::Entity(entity))
            .expect("entity retention after branch create")
            .pins
            .branch_pins,
        0
    );

    update_entity(&runtime, entity, "main");
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(RecordRef::Entity(entity))
            .expect("entity retention after main update")
            .pins
            .branch_pins,
        0
    );

    update_entity_on_branch(&runtime, entity, "feature", BranchId("feature".to_string()));
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(RecordRef::Entity(entity))
            .expect("entity retention after feature update")
            .pins
            .branch_pins,
        0
    );
}

#[test]
fn invalid_store_path_does_not_relabel_performed_in_memory_publication() {
    let root_path = unique_test_store_path("worth-relational-bad-store");
    std::fs::write(&root_path, b"not-a-directory").unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: root_path.clone(),
            segment_commit_capacity: 2,
        })
        .build();

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        batch_create("fail-closed"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let durability_deferred = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("the performed movement is not falsely acknowledged as durable");
    match &durability_deferred {
        TransactionCommitError::PerformedButDurabilityDeferred { error, .. } => {
            assert_eq!(
                error.stage,
                crate::publication::bundle::PublicationStage::DurableAppend
            );
            assert!(!error.detail.is_empty());
        }
        error => panic!("performed durability fault has the exact typed posture: {error:?}"),
    }
    assert!(!durability_deferred
        .commit_log()
        .events()
        .iter()
        .any(|event| {
            matches!(
                event,
                crate::transactions::data::CommitTraceEvent::CommitRejected { .. }
            )
        }));

    let performed_commit = durability_deferred
        .performed_commit()
        .expect("deferred error carries exact performed receipt");
    assert_eq!(
        runtime
            .history()
            .immutable_commit_receipt(performed_commit.commit_id),
        Some(performed_commit.clone())
    );
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".to_owned())),
        Some(performed_commit.clone())
    );
    let basis = crate::tests::support::test_owner_main_basis(&runtime)
        .expect("the performed branch basis remains admissible");
    let truth = runtime
        .read_truth()
        .read_observation(&basis.observation())
        .expect("the finalized performed root is readable");
    assert_eq!(truth.entities().len(), 1);
    assert_eq!(
        runtime
            .publication()
            .latest_bundle()
            .expect("in-memory publication finalization installs the bundle")
            .commit
            .commit_id,
        performed_commit.commit_id
    );
    assert_eq!(runtime.retention().inspect_plan().branch_pinned_entities, 0);
}
