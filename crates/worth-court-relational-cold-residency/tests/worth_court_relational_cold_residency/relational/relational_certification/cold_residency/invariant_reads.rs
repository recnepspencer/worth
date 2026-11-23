//! Native invariant reads preserve public results and storage failure semantics.
use super::invariant_source::recovered;
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    durability::*,
    storage::RelationalStorageUnavailableCause as Cause,
    transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
    },
};
fn storage_cause(error: TransactionCommitError) -> RelationalExecutionDenialCause {
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("invariant read preserves its execution storage failure: {error:?}")
    };
    let CommitExecutionDenialKind::Cause(cause) = denial.kind;
    cause
}

#[test]
fn selected_branch_views_preserve_cold_probe_and_demand() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, counter, entity, _) = recovered(execution, false);
    let main_branch = worth_relational::facade::history::BranchId("main".into());
    assert_eq!(
        runtime
            .observe_branch_region_residency(&main_branch, entity.partition_id)
            .unwrap()
            .full,
        RelationalRegionProbeStatus::Cold
    );
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(runtime
        .entity_record_at_observation(execution, &basis.observation(), entity)
        .unwrap()
        .is_some());
    assert_eq!(
        runtime
            .observe_branch_region_residency(&main_branch, entity.partition_id)
            .unwrap()
            .full,
        RelationalRegionProbeStatus::Resident
    );
    assert!(counter.reads.load(Ordering::Relaxed) > 0);
}

#[test]
fn invariant_storage_refusal_is_sticky_and_preserves_publication() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, counter, entity, medium) = recovered(execution, true);
    let before = runtime.observe_complexity_counters();
    let frontier = runtime.history().latest_patch_stream_position();
    let first = storage_cause(
        runtime
            .validation()
            .mutation_sensitive_state(execution)
            .unwrap_err(),
    );
    assert_eq!(
        first,
        RelationalExecutionDenialCause::StorageUnavailable(
            worth_relational::facade::storage::RelationalStorageUnavailable {
                subject: worth_relational::facade::storage::RelationalStorageSubject::Region(
                    entity.partition_id
                ),
                cause: Cause::IntegrityMismatch,
            },
        )
    );
    let reads = counter.reads.load(Ordering::Relaxed);
    assert!(reads > 0);
    counter.damaged.store(false, Ordering::Relaxed);
    assert_eq!(
        storage_cause(
            runtime
                .validation()
                .mutation_sensitive_state(execution)
                .unwrap_err()
        ),
        first
    );
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
    let after = runtime.observe_complexity_counters();
    assert_eq!(
        after.preparation_packet_count,
        before.preparation_packet_count
    );
    assert_eq!(
        after.invariant_entity_slot_scans,
        before.invariant_entity_slot_scans
    );
    assert_eq!(
        after.invariant_authoritative_entity_records_materialized,
        before.invariant_authoritative_entity_records_materialized
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    assert!(medium.offered().is_empty());
}

#[test]
fn cold_and_warm_invariant_reads_preserve_results_without_reloading() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, counter, _, _) = recovered(execution, false);
    let before = runtime.observe_complexity_counters();
    let cold = runtime
        .validation()
        .mutation_sensitive_state(execution)
        .unwrap();
    assert!(
        !cold.results().is_empty(),
        "the fixture must execute a real native rule"
    );
    let after_cold = runtime.observe_complexity_counters();
    let reads = counter.reads.load(Ordering::Relaxed);
    assert!(reads > 0);
    let warm = runtime
        .validation()
        .mutation_sensitive_state(execution)
        .unwrap();
    assert_eq!(cold.results(), warm.results());
    let after_warm = runtime.observe_complexity_counters();
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
    let cold_packets = after_cold.preparation_packet_count - before.preparation_packet_count;
    let warm_packets = after_warm.preparation_packet_count - after_cold.preparation_packet_count;
    assert!(cold_packets > 0);
    assert_eq!(
        cold_packets, warm_packets,
        "a discarded round must not publish its counters"
    );
    assert_eq!(
        after_cold.invariant_entity_slot_scans - before.invariant_entity_slot_scans,
        after_warm.invariant_entity_slot_scans - after_cold.invariant_entity_slot_scans,
    );
}

#[test]
fn the_invariant_orchestrator_uses_head_while_full_history_failure_stays_typed() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, counter, entity, medium) = recovered(execution, false);
    let current_members = counter.members.keys().copied().collect::<Vec<_>>();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current_members,
                source: Some(counter.clone()),
            },
        )
        .unwrap();
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            checkpoint,
        )),
        vec![],
        vec![],
        counter.clone(),
    );
    let manifest = super::selected_directory::read(&input);
    let history = manifest
        .mirror
        .iter()
        .find(|region| region.partition_id == entity.partition_id)
        .unwrap()
        .history;
    assert_eq!(counter.reads.load(Ordering::Relaxed), 0);
    *counter.failed_member.lock().unwrap() = Some(history);
    let position = runtime.history().latest_patch_stream_position();
    let cold = runtime
        .validation()
        .mutation_sensitive_state(execution)
        .unwrap();
    assert!(!cold.results().is_empty());
    let reads = counter.reads.load(Ordering::Relaxed);
    assert!(reads > 0);
    assert!(!counter.member_reads.lock().unwrap().contains(&history));
    let warm = runtime
        .validation()
        .mutation_sensitive_state(execution)
        .unwrap();
    assert_eq!(cold.results(), warm.results());
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
    let full = runtime
        .storage_access()
        .storage_stats(execution)
        .unwrap_err();
    assert_eq!(
        full,
        RelationalExecutionDenialCause::StorageUnavailable(
            worth_relational::facade::storage::RelationalStorageUnavailable {
                subject: worth_relational::facade::storage::RelationalStorageSubject::Region(
                    entity.partition_id
                ),
                cause: Cause::Damaged,
            },
        )
    );
    let reads = counter.reads.load(Ordering::Relaxed);
    *counter.failed_member.lock().unwrap() = None;
    assert_eq!(
        runtime
            .storage_access()
            .storage_stats(execution)
            .unwrap_err(),
        full
    );
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert!(medium.offered().is_empty());
}
