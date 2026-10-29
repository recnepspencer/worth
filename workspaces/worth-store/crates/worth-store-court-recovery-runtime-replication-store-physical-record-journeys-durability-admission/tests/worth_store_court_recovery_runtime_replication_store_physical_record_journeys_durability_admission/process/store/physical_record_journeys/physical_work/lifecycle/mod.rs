#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::lifecycle::*;
use {std::collections::BTreeSet};

use {tempfile::tempdir};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::PhysicalSignalShutdownOutcome, worth_store::physical_runtime::PhysicalWorkSubmissionDenial, worth_store::physical_runtime::PhysicalWorkSubmissionOutcome, worth_store::physical_runtime::PhysicalWorkSubmissionStale, worth_store::physical_runtime::PhysicalWorkTerminalDisposition, worth_store::physical_runtime::PhysicalWorkTerminalStage};

use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_mutation_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

#[test]
fn retained_submission_handles_stale_and_shutdown_classifies_every_identity() {
    let root = tempdir().unwrap();
    let (profile, read_request, mutation_request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile.clone());
    let first_runtime = serving.runtime_identity();
    let first_signal = serving.physical_signal_runtime_identity();
    let clock = serving.physical_signal_clock_observation().unwrap();
    assert_eq!(clock.current_tick(), 0);
    assert_eq!(clock.last_advance_ordinal(), 0);
    let read = serving.physical_read_submission();
    let mutation = serving.physical_mutation_submission();
    let read_receipt = success(read.submit(read_request.clone()));
    let mutation_receipt = success(mutation.submit(mutation_request.clone()));
    assert_ne!(read_receipt.identity(), mutation_receipt.identity());
    assert_eq!(
        read_receipt.identity().generation(),
        mutation_receipt.identity().generation()
    );

    let closed = serving.close();
    assert_eq!(closed.work().declared(), 2);
    assert_eq!(closed.work().terminal().len(), 2);
    assert_eq!(closed.work().residual(), 0);
    assert_eq!(closed.work().unaccounted_terminal(), 0);
    assert!(closed
        .work()
        .terminal()
        .windows(2)
        .all(|pair| pair[0].identity().operation() < pair[1].identity().operation()));
    for terminal in closed.work().terminal() {
        assert_eq!(terminal.stage(), worth_store::physical_runtime::PhysicalWorkTerminalStage::Declared);
        assert_eq!(
            terminal.disposition(),
            worth_store::physical_runtime::PhysicalWorkTerminalDisposition::ClosedBeforeReadiness
        );
    }
    assert_eq!(closed.signal(), worth_store::physical_runtime::PhysicalSignalShutdownOutcome::Disposed);
    assert!(matches!(
        read.submit(read_request.clone()).into_raw(),
        worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalWorkSubmissionStale::OwnerReleased)
    ));
    assert!(matches!(
        mutation.submit(mutation_request.clone()).into_raw(),
        worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalWorkSubmissionStale::OwnerReleased)
    ));

    let reopened = serving_from_open_with_work_profile(root.path(), profile);
    let reopened_receipt = success(reopened.physical_read_submission().submit(read_request));
    assert_ne!(reopened.runtime_identity(), first_runtime);
    assert_eq!(
        reopened_receipt.signal_profile(),
        read_receipt.signal_profile()
    );
    assert_ne!(reopened.physical_signal_runtime_identity(), first_signal);
    reopened.close();
}

#[test]
fn lifecycle_generation_advance_revokes_submission_before_owner_release() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let submission = serving.physical_read_submission();

    serving.certification_begin_lifecycle_termination();

    assert!(matches!(
        submission.submit(request).into_raw(),
        worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalWorkSubmissionStale::LifecycleGenerationAdvanced)
    ));
    assert_eq!(serving.close().work().declared(), 0);
}

#[test]
fn two_independent_mutation_handles_submit_disjoint_work_concurrently_without_branch_lanes() {
    let root = tempdir().unwrap();
    let (profile, first_request, second_request) = disjoint_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let first = serving.physical_mutation_submission();
    let second = serving.physical_mutation_submission();
    let gate = serving.certification_pause_physical_command_shards_after_lock();
    let receipts = std::thread::scope(|scope| {
        let first_join = scope.spawn(move || success(first.submit(first_request)));
        let second_join = scope.spawn(move || success(second.submit(second_request)));
        assert!(
            gate.await_arrivals(2),
            "both independent submissions must own distinct command shards concurrently"
        );
        gate.release();
        vec![first_join.join().unwrap(), second_join.join().unwrap()]
    });

    let identities: BTreeSet<_> = receipts
        .iter()
        .map(|receipt| receipt.identity().operation().get())
        .collect();
    assert_eq!(identities.len(), receipts.len());
    let closed = serving.close();
    assert_eq!(closed.work().declared(), receipts.len() as u64);
    assert_eq!(closed.work().terminal().len(), receipts.len());
    assert!(closed
        .work()
        .terminal()
        .windows(2)
        .all(|pair| pair[0].identity().operation() < pair[1].identity().operation()));
}

#[test]
fn unexpected_drop_publishes_terminal_work_without_retaining_authority() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let observer = serving.physical_work_observer();
    let submission = serving.physical_read_submission();
    let receipt = success(submission.submit(request.clone()));

    drop(serving);

    let terminal = observer
        .terminal()
        .expect("unexpected drop must publish physical work terminal truth");
    assert_eq!(terminal.declared(), 1);
    assert_eq!(terminal.terminal()[0].identity(), receipt.identity());
    assert_eq!(
        terminal.terminal()[0].disposition(),
        worth_store::physical_runtime::PhysicalWorkTerminalDisposition::DroppedBeforeReadiness
    );
    assert!(matches!(
        submission.submit(request).into_raw(),
        worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalWorkSubmissionStale::OwnerReleased)
    ));
}

#[test]
fn abort_classifies_declared_work_and_revokes_retained_submission() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let submission = serving.physical_read_submission();
    let receipt = success(submission.submit(request.clone()));

    let aborted = serving.abort();
    assert_eq!(aborted.work().declared(), 1);
    assert_eq!(aborted.work().residual(), 0);
    assert_eq!(aborted.work().unaccounted_terminal(), 0);
    assert_eq!(aborted.work().terminal()[0].identity(), receipt.identity());
    assert_eq!(
        aborted.work().terminal()[0].disposition(),
        worth_store::physical_runtime::PhysicalWorkTerminalDisposition::AbortedBeforeReadiness
    );
    assert!(matches!(
        submission.submit(request).into_raw(),
        worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalWorkSubmissionStale::OwnerReleased)
    ));
}

#[test]
fn close_race_accounts_every_success_and_stales_every_loser() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let submission = serving.physical_read_submission();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(9));

    let (closed, outcomes) = std::thread::scope(|scope| {
        let close_barrier = std::sync::Arc::clone(&barrier);
        let close = scope.spawn(move || {
            close_barrier.wait();
            serving.close()
        });
        let mut submissions = Vec::new();
        for _ in 0..8 {
            let handle = submission.clone();
            let request = request.clone();
            let submit_barrier = std::sync::Arc::clone(&barrier);
            submissions.push(scope.spawn(move || {
                submit_barrier.wait();
                handle.submit(request).into_raw()
            }));
        }
        let outcomes = submissions
            .into_iter()
            .map(|join| join.join().unwrap())
            .collect::<Vec<_>>();
        (close.join().unwrap(), outcomes)
    });

    let succeeded = outcomes
        .iter()
        .filter(|outcome| matches!(outcome, worth_proof::TransitionOutcome::Success(_)))
        .count();
    assert!(outcomes.iter().all(|outcome| matches!(
        outcome,
        worth_proof::TransitionOutcome::Success(_)
            | worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalWorkSubmissionStale::AdmissionStopped)
            | worth_proof::TransitionOutcome::Stale(worth_store::physical_runtime::PhysicalWorkSubmissionStale::OwnerReleased)
    )));
    assert_eq!(closed.work().declared(), succeeded as u64);
    assert_eq!(closed.work().terminal().len(), succeeded);
}

#[test]
fn uninstalled_contract_is_denied_before_command_admission() {
    let root = tempdir().unwrap();
    let (_, request, _) = work_fixture();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(root.path());
    assert!(matches!(
        serving
            .physical_read_submission()
            .submit(request)
            .into_raw(),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalWorkSubmissionDenial::SemanticContractNotInstalled)
    ));
    assert_eq!(serving.close().work().declared(), 0);
}
