use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::tests::support::*;

#[test]
fn canonical_consumers_resolve_around_the_real_root_to_route_cutover() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "handoff-anchor");
    let identity = runtime.main_branch_identity();
    let (_, old_basis) = runtime
        .observe_branch(&identity)
        .expect("old public basis is observable");
    let old_commit = old_basis
        .observation()
        .canonical_commit()
        .expect("seed publication is canonical")
        .commit
        .commit_id;

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("handoff-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("handoff candidate prepares");
    let publication_cell = candidate.publication_cell_for_test();
    let publication_gate = Arc::clone(publication_cell.coordination());
    let held_gate = publication_gate.enter();
    let port = runtime.publication_port();
    let publisher_done = Arc::new(AtomicBool::new(false));
    let publisher_done_signal = Arc::clone(&publisher_done);
    let publisher = std::thread::spawn(move || {
        let outcome = port.compare_and_publish(candidate);
        publisher_done_signal.store(true, Ordering::Release);
        outcome
    });

    let wait_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while publication_gate.wait_count() == 0 && std::time::Instant::now() < wait_deadline {
        std::thread::yield_now();
    }
    assert_eq!(publication_gate.wait_count(), 1);

    let observations = Arc::new(AtomicU64::new(0));
    let observation_count = Arc::clone(&observations);
    let reader_ready = Arc::new(AtomicBool::new(false));
    let reader_ready_signal = Arc::clone(&reader_ready);
    let observed_identity = identity.clone();
    let reader = std::thread::spawn(move || {
        let mut observed_commits = Vec::new();
        observe_canonical_consumers(&runtime, &observed_identity, &mut observed_commits);
        observation_count.fetch_add(1, Ordering::Release);
        assert_eq!(
            runtime
                .fork()
                .expect_err("fork rejects instead of waiting on an active publication"),
            crate::runtime::RelationalRuntimeForkDenial::PublicationInFlight
        );
        let checkpoint_error = runtime
            .durability_authority()
            .checkpoint()
            .expect_err("checkpoint rejects instead of waiting on an active publication");
        assert_eq!(
            checkpoint_error.class,
            crate::durability::data::RecoveryFailureClass::CheckpointPublicationInFlight
        );
        reader_ready_signal.store(true, Ordering::Release);

        while !publisher_done.load(Ordering::Acquire) {
            observe_canonical_consumers(&runtime, &observed_identity, &mut observed_commits);
            observation_count.fetch_add(1, Ordering::Release);
            std::thread::yield_now();
        }
        observe_canonical_consumers(&runtime, &observed_identity, &mut observed_commits);
        (runtime, observed_commits)
    });

    let observation_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while !reader_ready.load(Ordering::Acquire) && std::time::Instant::now() < observation_deadline
    {
        std::thread::yield_now();
    }
    assert!(reader_ready.load(Ordering::Acquire));
    assert!(observations.load(Ordering::Acquire) > 0);
    drop(held_gate);

    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        publisher.join().expect("publisher joins")
    else {
        panic!("publication must perform through the production path");
    };
    let (runtime, observed_commits) = reader.join().expect("public reader joins");
    let new_commit = performed.canonical_commit().commit.commit_id;
    assert_ne!(old_commit, new_commit);
    assert!(observed_commits
        .iter()
        .all(|commit| *commit == old_commit || *commit == new_commit));
    assert!(observed_commits.contains(&old_commit));
    assert!(observed_commits.contains(&new_commit));

    let stream = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest::default())
        .expect("performed stream is visible before settlement");
    assert!(stream
        .patches
        .iter()
        .any(|patch| patch.position == performed.patch_position()));
    let checkpoint_error = runtime
        .durability_authority()
        .checkpoint()
        .expect_err("performed publication still requires settlement");
    assert_eq!(
        checkpoint_error.class,
        crate::durability::data::RecoveryFailureClass::PerformedPublicationRequiresSettlement
    );
    let committed = runtime
        .settle_performed_publication(performed)
        .expect("performed publication settles explicitly");
    release_test_commit_snapshot(&runtime, &committed);
    runtime
        .durability_authority()
        .checkpoint()
        .expect("checkpoint admits after settlement");
}

fn observe_canonical_consumers(
    runtime: &crate::facade::runtime::RelationalRuntime,
    identity: &crate::branch::RelationalBranchIdentity,
    observed_commits: &mut Vec<crate::facade::history::CommitId>,
) {
    let (_, basis) = runtime
        .observe_branch(identity)
        .expect("concurrent public observation remains complete");
    let observation = basis.observation();
    let envelope = observation
        .canonical_commit()
        .expect("observed root carries one canonical commit");
    let commit_id = envelope.commit.commit_id;
    assert_eq!(
        runtime.history().immutable_commit_receipt(commit_id),
        Some(envelope.commit.clone())
    );
    assert_eq!(
        runtime.replay().canonical_commit_envelope(commit_id),
        Some(envelope.clone())
    );
    let stream = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest::default())
        .expect("canonical patch stream remains readable");
    assert!(stream
        .patches
        .iter()
        .any(|patch| patch.authoritative_record_patches
            == envelope.patch.authoritative_record_patches));
    observed_commits.push(commit_id);
}
