//! Recovered nonhead bodies are retained by readers, never by the catalog.
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[test]
fn recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (recovery, source, _) = fixture_with_history(execution, false, 16);
    source.damaged.store(false, Ordering::Relaxed);
    let position = *source.commits.keys().nth(3).unwrap();
    let mut runtime = durable_runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(position)
        .unwrap();
    assert_ne!(
        runtime
            .history()
            .branch_head(&BranchId("main".into()))
            .unwrap()
            .commit_id,
        receipt.commit_id
    );
    let read = || {
        runtime
            .history()
            .demand_commit_envelope(execution, receipt.commit_id)
    };
    let first = read().unwrap().unwrap();
    assert_eq!(first.commit, receipt);
    let reads = source.commit_reads.lock().unwrap().len();
    source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Commit(position), Cause::SourceUnavailable);
    let simultaneous = read().unwrap().unwrap();
    assert!(
        Arc::ptr_eq(&first, &simultaneous),
        "in-flight demand shares the admitted body"
    );
    assert_eq!(source.commit_reads.lock().unwrap().len(), reads);
    let weak = Arc::downgrade(&first);
    drop(first);
    assert!(
        weak.upgrade().is_some(),
        "the second reader still has custody"
    );
    drop(simultaneous);
    assert!(
        weak.upgrade().is_none(),
        "a catalog index must not retain an old queried body"
    );
    let frontier = runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(&runtime).offered().len();
    assert_eq!(
        read().unwrap_err(),
        Execution::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(position),
            cause: Cause::SourceUnavailable,
        })
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
    source.failures.lock().unwrap().clear();
    assert_eq!(read().unwrap().unwrap().commit, receipt);
}

#[test]
fn settled_live_nonhead_envelopes_release_only_when_a_durable_copy_exists() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for durable in [false, true] {
        let runtime = if durable {
            durable_runtime_with_test_schema()
        } else {
            runtime_with_test_schema()
        };
        let entity = create_entity(execution, &runtime, "live-envelope-before");
        let commit = runtime
            .history()
            .branch_head(&BranchId("main".into()))
            .unwrap()
            .commit_id;
        let body = runtime
            .history()
            .demand_commit_envelope(execution, commit)
            .unwrap()
            .unwrap();
        let weak = Arc::downgrade(&body);
        drop(body);
        update_entity_and_release_snapshot(execution, &runtime, entity, "live-envelope-after");
        runtime.history.reclaim_retired_branch_roots(1024);
        assert!(
            runtime.history.retired_branch_root_count() < 4,
            "a single retired version fits reclamation headroom"
        );
        if durable {
            assert!(
                weak.upgrade().is_none(),
                "a settled nonhead without a reader must release its journal-backed body"
            );
        } else {
            assert!(
                weak.upgrade().is_some(),
                "memory-only history has no other copy"
            );
        }
        let patches = runtime
            .history()
            .bounded_canonical_commit_patches(
                execution,
                commit,
                std::num::NonZeroUsize::new(32).unwrap(),
            )
            .unwrap();
        assert!(
            !patches.is_empty(),
            "released bodies read back from their durable journal"
        );
    }
}

#[test]
fn cold_source_is_bound_before_a_durable_runtime_publishes_its_first_commit() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let source = runtime
        .durability
        .cold_source()
        .expect("medium attachment supplies its journal read port");
    let entity = create_entity(execution, &runtime, "attached-source-seed");
    let position = runtime.history().latest_patch_stream_position().unwrap();
    let bytes = source.read_commit(position, 64 << 20).unwrap();
    let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes).unwrap();
    assert_eq!(artifact.position(), position);
    let envelope =
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::readmit(
            artifact.as_bytes(),
            execution,
        )
        .unwrap()
        .into_positioned();
    assert_eq!(
        envelope.commit.commit_id,
        runtime
            .history()
            .branch_head(&BranchId("main".into()))
            .unwrap()
            .commit_id
    );
    assert!(envelope
        .patch
        .authoritative_record_patches
        .iter()
        .any(|patch| patch.target == crate::transactions::data::RecordRef::Entity(entity)));
    assert_eq!(
        runtime.history().latest_patch_stream_position(),
        Some(position)
    );
    assert!(runtime_with_test_schema()
        .durability
        .cold_source()
        .is_none());
}

#[test]
fn released_live_history_remains_in_external_capture_and_retained_recovery() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "live-index-seed");
    for n in 0..12 {
        update_entity_and_release_snapshot(execution, &runtime, entity, &format!("live-index-{n}"));
    }
    runtime.history.reclaim_retired_branch_roots(1024);
    let ids = runtime
        .history
        .commit_artifacts()
        .into_iter()
        .map(|artifact| artifact.commit_id())
        .collect::<Vec<_>>();
    let resident = runtime
        .history
        .commit_artifacts()
        .into_iter()
        .filter(|artifact| artifact.resident_envelope().is_some())
        .count();
    assert!(
        resident < 4,
        "released journal bodies fit head and reclamation headroom"
    );
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for id in &ids {
        assert_eq!(
            recovered
                .history()
                .demand_commit_envelope(execution, *id)
                .unwrap()
                .unwrap()
                .commit
                .commit_id,
            *id
        );
    }
    let (source, reads, _) = checkpoint_fixture(execution, runtime, entity, false);
    reads.damaged.store(false, Ordering::Relaxed);
    let mut reopened = durable_runtime_with_test_schema();
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for id in ids {
        assert_eq!(
            reopened
                .history()
                .demand_commit_envelope(execution, id)
                .unwrap()
                .unwrap()
                .commit
                .commit_id,
            id
        );
    }
    assert!(
        reads.commit_reads.lock().unwrap().len() < 32,
        "head admission and one selected read per commit have bounded headroom"
    );
}

#[test]
fn recovered_nonhead_tail_envelopes_release_their_actual_journal_backed_bodies() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (_, source, _) = fixture_with_history(execution, false, 8);
    source.damaged.store(false, Ordering::Relaxed);
    let tail = source.commits.values().map(|bytes| {
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap()
    }).collect();
    let recovery = RelationalDurableRecoverySource::new(None, tail, Vec::new(), source.clone());
    let position = *source.commits.keys().nth(2).unwrap();
    let mut runtime = durable_runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    runtime.history.reclaim_retired_branch_roots(1024);
    assert_eq!(
        source.commit_reads.lock().unwrap().len(),
        0,
        "tail replay already owns its readmitted frames"
    );
    let head = runtime
        .history()
        .branch_head(&BranchId("main".into()))
        .unwrap();
    assert!(runtime
        .history
        .commit_artifact(head.commit_id)
        .unwrap()
        .resident_envelope()
        .is_some());
    assert!(
        runtime
            .history
            .commit_artifacts()
            .iter()
            .all(|artifact| artifact.envelope_residency().resident_payload().is_none()),
        "journal placement releases encoded duplicate buffers too"
    );
    let receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(position)
        .unwrap();
    let body = runtime
        .history()
        .demand_commit_envelope(execution, receipt.commit_id)
        .unwrap()
        .unwrap();
    let weak = Arc::downgrade(&body);
    drop(body);
    assert!(
        weak.upgrade().is_none(),
        "replay must not turn a nonhead journal body into permanent memory truth"
    );
    assert_eq!(
        runtime
            .history()
            .demand_commit_envelope(execution, receipt.commit_id)
            .unwrap()
            .unwrap()
            .commit,
        receipt
    );
}
