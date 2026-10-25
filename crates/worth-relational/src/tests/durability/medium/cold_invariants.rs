//! Real cold recovery crosses the invariant orchestrator, including telemetry.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
use crate::validation::data::{
    InvariantCatalog, InvariantRegistration, InvariantRule, RecordKindTag,
};

fn recovered(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    damaged: bool,
) -> (
    crate::runtime::RelationalRuntime,
    Arc<MemberSource>,
    crate::identity::data::EntityId,
) {
    let (source, counter, entity) = fixture(execution, false);
    counter.damaged.store(damaged, Ordering::Relaxed);
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
                InvariantRule::LiveRecordRequiresSidecar(RecordKindTag::Entity),
            )],
        })
        .build_durable(TestDurableMedium::new().attachment());
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    (runtime, counter, entity)
}

fn storage_cause(error: TransactionCommitError) -> RelationalExecutionDenialCause {
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("invariant read preserves its execution storage failure: {error:?}")
    };
    let CommitExecutionDenialKind::Cause(cause) = denial.kind;
    cause
}

#[test]
fn selected_branch_views_preserve_cold_probe_and_demand() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, counter, entity) = recovered(execution, false);
    let transaction = test_owner_begin_transaction_for_main(&runtime);
    let selected = runtime.selected_branch_state(&transaction.basis).unwrap();
    let answer = crate::storage::residency::probe_full_sources(execution, [&selected], |[view]| {
        view.get_partition(entity.partition_id).is_some()
    })
    .unwrap();
    assert!(answer);
    assert!(counter.reads.load(Ordering::Relaxed) > 0);
}

#[test]
fn invariant_storage_refusal_is_sticky_and_does_not_settle_provisional_counts() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, counter, entity) = recovered(execution, true);
    let before = runtime.performance_access().counters();
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
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(
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
    let after = runtime.performance_access().counters();
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
    assert!(attached_test_medium(&runtime).offered().is_empty());
}

#[test]
fn cold_and_warm_invariant_reads_settle_one_round_of_evidence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (runtime, counter, _) = recovered(execution, false);
    let before = runtime.performance_access().counters();
    let cold = runtime
        .validation()
        .mutation_sensitive_state(execution)
        .unwrap();
    assert!(
        !cold.results().is_empty(),
        "the fixture must execute a real native rule"
    );
    let after_cold = runtime.performance_access().counters();
    let reads = counter.reads.load(Ordering::Relaxed);
    assert!(reads > 0);
    let warm = runtime
        .validation()
        .mutation_sensitive_state(execution)
        .unwrap();
    assert_eq!(cold.results(), warm.results());
    let after_warm = runtime.performance_access().counters();
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
