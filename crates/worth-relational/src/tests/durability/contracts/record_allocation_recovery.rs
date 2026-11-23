//! Record-allocation recovery courts: reused slot consumption, burned append
//! frontier, orphaned append reservation and orphaned reclaimed reservation.
//! Fixture adaptation (C2.4): restored from df2420390e / f9f15bf8c8 on the
//! Memory runtime (Embedded checkpoint plus tail recovery plan), since these
//! claims exercise record-identity allocator frontiers and tail replay without
//! physical-medium content.
use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn tail_replay_consumes_canonical_reused_record_allocation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(
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
    assert!(
        runtime
            .retention()
            .run_pass(execution)
            .unwrap()
            .entity_reclaimed
            <= 1
    );
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .reusable_entity_slots,
        1
    );
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();

    let replacement = create_entity(execution, &runtime, "allocation-after");
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema_profile(
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

    let runtime = runtime_with_test_schema();
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
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();

    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
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

    let runtime = runtime_with_test_schema();
    let original = create_entity(execution, &runtime, "orphan-append-root");
    let mut pending = runtime.record_identity.begin_allocations();
    let reservation = pending
        .reserve(
            crate::history::data::RecordAllocationClass::Entity,
            original.partition_id,
        )
        .unwrap();
    assert_eq!(reservation.slot as u64, original.local_slot.0 + 1);
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    drop(pending);

    let mut recovered = runtime_with_test_schema();
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

    let runtime = runtime_with_test_schema_profile(
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
    runtime.retention().run_pass(execution).unwrap();
    let mut pending = runtime.record_identity.begin_allocations();
    let reservation = pending
        .reserve(
            crate::history::data::RecordAllocationClass::Entity,
            original.partition_id,
        )
        .unwrap();
    assert_eq!(reservation.slot as u64, original.local_slot.0);
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    drop(pending);

    let mut recovered = runtime_with_test_schema_profile(
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
