use super::*;

#[test]
fn historical_inspection_stays_branch_local_under_divergence_and_reclaim_pressure() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .visibility_cache_policy(VisibilityCachePolicy {
            enabled: true,
            protect_branch_heads: false,
            protect_replay_retained: false,
            protect_active_snapshots: false,
            recent_version_window: 0,
        })
        .build();
    let created = create_entity_outcome(execution, &runtime, "base");
    let entity = changed_entities(&created)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch");

    let main_update = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("main-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "main",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(execution, &runtime).expect("main update")
    };
    let feature_update = {
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
        txn.commit(execution, &runtime).expect("feature update")
    };

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&main_update.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&feature_update.snapshot)
        .is_ok());

    let retained_only = runtime
        .inspect_what_happened()
        .open_historical_view(
            execution,
            created.version_id,
            HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");
    assert!(retained_only.view.is_none());
    assert_eq!(
        retained_only.availability,
        InspectionAvailability::UnavailableByRetention
    );

    let main = reconstructed_record_inspection(
        execution,
        &runtime,
        &BranchId("main".to_string()),
        created.version_id,
        crate::facade::transactions::RecordRef::Entity(entity),
    );
    let feature = reconstructed_record_inspection(
        execution,
        &runtime,
        &BranchId("feature".to_string()),
        created.version_id,
        crate::facade::transactions::RecordRef::Entity(entity),
    );

    assert_eq!(
        main.record_observation.availability,
        InspectionAvailability::Reconstructed
    );
    assert_eq!(
        feature.record_observation.availability,
        InspectionAvailability::Reconstructed
    );
    assert_eq!(
        main.aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("main".to_string()))
    );
    assert_eq!(
        feature
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("feature".to_string()))
    );
    assert_ne!(
        main.aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.entries.len()),
        feature
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.entries.len())
    );
}

#[test]
fn recent_commit_inspection_and_branch_head_reads_stay_branch_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let base = create_entity_outcome(execution, &runtime, "base");
    let entity = changed_entities(&base)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch");

    let main_update = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("main-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "main",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(execution, &runtime).expect("main update")
    };
    let feature_update = {
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
        txn.commit(execution, &runtime).expect("feature update")
    };

    let feature_head = runtime
        .inspect_what_happened()
        .inspect_branch_head(execution, &BranchId("feature".to_string()))
        .unwrap()
        .expect("feature branch head");
    let feature_window =
        recent_commit_window(execution, &runtime, &BranchId("feature".to_string()), 8);
    let main_window = recent_commit_window(execution, &runtime, &BranchId("main".to_string()), 8);

    assert_eq!(
        feature_head.commit.branch_id,
        BranchId("feature".to_string())
    );
    assert_eq!(
        feature_head.commit.commit_id,
        feature_update.commit.commit_id
    );
    assert_eq!(
        feature_window
            .branch_head
            .as_ref()
            .map(|head| head.commit_id),
        Some(feature_update.commit.commit_id)
    );
    assert!(feature_window
        .commits
        .iter()
        .all(|inspection| inspection.commit.branch_id == BranchId("feature".to_string())));
    assert!(main_window
        .commits
        .iter()
        .all(|inspection| inspection.commit.branch_id == BranchId("main".to_string())));
    assert!(main_window
        .commits
        .iter()
        .any(|inspection| inspection.commit.commit_id == main_update.commit.commit_id));
    assert!(!feature_window
        .commits
        .iter()
        .any(|inspection| inspection.commit.commit_id == main_update.commit.commit_id));
}
