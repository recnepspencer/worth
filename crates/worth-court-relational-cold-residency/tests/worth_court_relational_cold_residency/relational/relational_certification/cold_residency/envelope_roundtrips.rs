//! Original nonhead history remains readable across real journal and member recovery.
use super::{current_publications::create_entity, envelope_source::reopened};
use std::{num::NonZeroUsize, sync::atomic::Ordering};
use worth_relational::facade::{config::RelationalRuntimeProfile, durability::*};
use worth_testkit_relational::{
    cold_recovery::{capture_source, update_entity},
    durable_medium::TestDurableMedium,
    durable_publication::{durable_runtime, memory_runtime},
};
#[test]
fn live_nonhead_history_reads_original_patches_from_memory_and_durable_owners() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    for durable in [false, true] {
        let medium = TestDurableMedium::new();
        let runtime = if durable {
            durable_runtime(&medium)
        } else {
            memory_runtime()
        };
        let entity = create_entity(request, &runtime, "live-envelope-before");
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let commit = basis.observation().commit_id().unwrap();
        drop(basis);
        let body = runtime
            .replay()
            .canonical_commit_envelope(request, commit)
            .unwrap()
            .unwrap();
        drop(body);
        update_entity(request, &runtime, entity, "live-envelope-after");
        runtime.run_branch_root_reclamation_pass();
        assert!(
            runtime.retired_branch_root_count() < 4,
            "a single retired version fits reclamation headroom"
        );
        let patches = runtime
            .history()
            .bounded_canonical_commit_patches(request, commit, NonZeroUsize::new(32).unwrap())
            .unwrap();
        assert!(
            !patches.is_empty(),
            "original bodies remain readable from their actual owner"
        );
    }
}
#[test]
fn live_history_remains_in_external_capture_and_retained_recovery() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(request, &runtime, "live-index-seed");
    for n in 0..12 {
        update_entity(request, &runtime, entity, &format!("live-index-{n}"));
    }
    runtime.run_branch_root_reclamation_pass();
    let ids = medium
        .committed_artifacts()
        .unwrap()
        .iter()
        .map(RelationalCanonicalCommitArtifact::commit_id)
        .collect::<Vec<_>>();
    let plan = runtime
        .durability()
        .recovery_plan(
            request,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let receiving_medium = TestDurableMedium::new();
    let mut recovered = durable_runtime(&receiving_medium);
    recovered
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    for id in &ids {
        assert_eq!(
            recovered
                .replay()
                .canonical_commit_envelope(request, *id)
                .unwrap()
                .unwrap()
                .commit
                .commit_id,
            *id
        );
    }
    let original = capture_source(request, runtime, medium, entity, false);
    let receiving_medium = TestDurableMedium::new();
    let mut receiver = durable_runtime(&receiving_medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            request,
            original.input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiver
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    for id in ids {
        assert_eq!(
            receiver
                .replay()
                .canonical_commit_envelope(request, id)
                .unwrap()
                .unwrap()
                .commit
                .commit_id,
            id
        );
    }
    assert!(
        original.source.commit_reads.lock().unwrap().len() < 32,
        "head admission and one selected read per commit have bounded headroom"
    );
}
#[test]
fn recovered_nonhead_tail_envelopes_keep_their_original_receipt_and_source_read_cut() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(
        request,
        8,
        true,
        RelationalRuntimeProfile::CertificationCore,
    );
    fixture.runtime.run_branch_root_reclamation_pass();
    let position = *fixture.source.commits.keys().nth(2).unwrap();
    assert_eq!(
        fixture.source.commit_reads.lock().unwrap().len(),
        0,
        "tail replay already owns its readmitted frames"
    );
    let receipt = fixture
        .runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(position)
        .unwrap();
    let body = fixture
        .runtime
        .replay()
        .canonical_commit_envelope(request, receipt.commit_id)
        .unwrap()
        .unwrap();
    drop(body);
    assert_eq!(
        fixture
            .runtime
            .replay()
            .canonical_commit_envelope(request, receipt.commit_id)
            .unwrap()
            .unwrap()
            .commit,
        receipt
    );
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        0,
        "Journal-only reconstruction uses no selected member"
    );
}
