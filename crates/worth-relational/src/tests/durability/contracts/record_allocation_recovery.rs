use super::*;

#[test]
fn tail_replay_consumes_canonical_reused_record_allocation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    let created = create_entity_outcome(execution, &runtime, "allocation-before");
    let original = changed_entities(&created)[0];
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    let deleted = delete_entity(execution, &runtime, original);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    assert!(runtime.retention().run_pass().entity_reclaimed <= 1);
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats()
            .reusable_entity_slots,
        1
    );
    runtime.durability_authority().checkpoint().unwrap();

    let replacement = create_entity(execution, &runtime, "allocation-after");
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(replacement.local_slot, original.local_slot);
    assert!(replacement.generation.0 > original.generation.0);
    let current = recovered
        .read_truth()
        .read_version(execution, recovered.current_version_id())
        .expect("read request remains live");
    assert_eq!(
        read_entity_name(current.get_entity(replacement).unwrap()),
        Some("allocation-after".into())
    );
    let historical = recovered
        .read_truth()
        .read_version(execution, created.version_id)
        .expect("read request remains live");
    assert_eq!(
        read_entity_name(historical.get_entity(original).unwrap()),
        Some("allocation-before".into())
    );
}

#[test]
fn checkpoint_restores_burned_append_frontier_beyond_all_retained_roots() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let original = create_entity(execution, &runtime, "frontier-root");
    let mut burned = runtime.record_identity.begin_allocations();
    let reservation = burned
        .reserve(
            crate::history::data::RecordAllocationClass::Entity,
            original.partition_id,
        )
        .unwrap();
    assert_eq!(reservation.slot as u64, original.local_slot.0 + 1);
    drop(burned);
    runtime.durability_authority().checkpoint().unwrap();

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let after_recovery = create_entity(execution, &recovered, "frontier-after-recovery");

    assert_eq!(after_recovery.local_slot.0, original.local_slot.0 + 2);
    assert_eq!(after_recovery.generation.0, 1);
}

#[test]
fn recovery_releases_orphaned_append_reservation_without_reusing_its_gap() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let original = create_entity(execution, &runtime, "orphan-append-root");
    let mut pending = runtime.record_identity.begin_allocations();
    let reservation = pending
        .reserve(
            crate::history::data::RecordAllocationClass::Entity,
            original.partition_id,
        )
        .unwrap();
    assert_eq!(reservation.slot as u64, original.local_slot.0 + 1);
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    drop(pending);

    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert!(recovered.record_identity.pending_snapshot().is_empty());
    let after_recovery = create_entity(execution, &recovered, "orphan-append-after");
    assert_eq!(after_recovery.local_slot.0, original.local_slot.0 + 2);
}

#[test]
fn recovery_returns_orphaned_reclaimed_reservation_to_reuse() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    let created = create_entity_outcome(execution, &runtime, "orphan-reclaimed-root");
    let original = changed_entities(&created)[0];
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    let deleted = delete_entity(execution, &runtime, original);
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    runtime.retention().run_pass();
    let mut pending = runtime.record_identity.begin_allocations();
    let reservation = pending
        .reserve(
            crate::history::data::RecordAllocationClass::Entity,
            original.partition_id,
        )
        .unwrap();
    assert_eq!(reservation.slot as u64, original.local_slot.0);
    runtime.durability_authority().checkpoint().unwrap();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    drop(pending);

    let mut recovered = persisted_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert!(recovered.record_identity.pending_snapshot().is_empty());
    let replacement = create_entity(execution, &recovered, "orphan-reclaimed-after");
    assert_eq!(replacement.local_slot, original.local_slot);
    assert!(replacement.generation.0 > original.generation.0);
}

#[test]
fn tail_replay_rejects_canonical_allocation_targeting_a_live_slot() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let original = create_entity(execution, &runtime, "allocation-live");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity(execution, &runtime, "allocation-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let tail = plan
        .tail_log
        .last_mut()
        .expect("tail commit is selected")
        .envelope_mut_for_test();
    let forged = crate::identity::data::EntityId::new(
        original.partition_id,
        original.local_slot.0,
        original.generation.0.saturating_add(1),
    );
    tail.install_record_allocations(vec![
        crate::history::data::CanonicalRecordAllocation::with_origin(
            0,
            crate::transactions::data::RecordRef::Entity(forged),
            crate::history::data::RecordAllocationOrigin::Reclaimed {
                prior_generation: original.generation.0,
            },
        ),
    ]);

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("replay must not overwrite a live checkpoint slot");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert!(error.detail.contains("selects unavailable Entity"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn tail_replay_rejects_missing_canonical_allocation_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity(execution, &runtime, "allocation-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    create_entity(execution, &runtime, "allocation-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.tail_log
        .last_mut()
        .expect("tail commit is selected")
        .envelope_mut_for_test()
        .install_record_allocations(Vec::new());

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("replay must require canonical allocation evidence");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert!(error.detail.contains("missing ordinal 0"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn tail_replay_gap_admission_cannot_replace_a_different_canonical_effect() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let checkpoint_entity = create_entity(execution, &runtime, "allocation-gap-checkpoint");
    runtime.durability_authority().checkpoint().unwrap();
    let tail_entity = create_entity(execution, &runtime, "allocation-gap-tail");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let forged_entity = crate::identity::data::EntityId::new(
        tail_entity.partition_id,
        checkpoint_entity.local_slot.0 + 4,
        tail_entity.generation.0,
    );
    plan.tail_log
        .last_mut()
        .expect("tail commit")
        .envelope_mut_for_test()
        .install_record_allocations(vec![
            crate::history::data::CanonicalRecordAllocation::with_origin(
                0,
                crate::transactions::data::RecordRef::Entity(forged_entity),
                crate::history::data::RecordAllocationOrigin::AppendFrontier,
            ),
        ]);

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("gap admission cannot authorize a different replayed effect");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert!(error.detail.contains("canonical effect"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn metadata_only_merge_rejects_unconsumable_allocation_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "allocation-shared");
    create_branch_from_main(&runtime, "allocation-feature");
    delete_entity(execution, &runtime, entity);
    delete_entity_on_branch(
        execution,
        &runtime,
        entity,
        BranchId("allocation-feature".to_owned()),
    );
    runtime.durability_authority().checkpoint().unwrap();
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_owned()),
                source_branch: BranchId("allocation-feature".to_owned()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .unwrap();
    runtime.execute_prepared_merge(execution, prepared).unwrap();
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let tail = plan
        .tail_log
        .last_mut()
        .expect("merge tail is selected")
        .envelope_mut_for_test();
    tail.install_record_allocations(vec![crate::history::data::CanonicalRecordAllocation::new(
        0,
        crate::transactions::data::RecordRef::Entity(entity),
    )]);

    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("a metadata-only merge cannot carry allocation evidence");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert!(error.detail.contains("without a mutation replay path"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}
