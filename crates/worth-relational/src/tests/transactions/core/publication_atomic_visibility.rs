use crate::facade::history::CommitId;
use crate::tests::support::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

#[test]
fn concurrent_reference_readers_observe_only_complete_old_or_new_roots() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "atomic-reader-anchor");
    let old_basis = crate::tests::support::test_owner_main_basis(&runtime).expect("old basis");
    let old_root_id = old_basis.descriptor().root_identity();

    let mut transaction = runtime
        .begin_branch_transaction(
            &old_basis,
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("reader-race transaction binds");
    transaction
        .push_batch(
            batch_create("atomic-reader-new-root"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("reader-race candidate prepares");
    let publication_cell = candidate.publication_cell_for_test();
    let publication_gate = Arc::clone(publication_cell.coordination());
    let held_gate = publication_gate.enter();

    let identity = runtime.main_branch_identity();
    let (old_descriptor, old_observation) = runtime
        .observe_branch(&identity)
        .expect("public observation sees the complete prior root");
    assert_eq!(old_descriptor.root_identity(), old_root_id);
    assert_public_observation_is_complete(&old_descriptor, &old_observation);

    let reader_started = Arc::new(AtomicBool::new(false));
    let reader_iterations = Arc::new(AtomicU64::new(0));
    let reader_done = Arc::clone(&reader_started);
    let iteration_counter = Arc::clone(&reader_iterations);
    let publisher_done = Arc::new(AtomicBool::new(false));
    let publisher_done_signal = Arc::clone(&publisher_done);
    let (publication_finished, publication_completion) = std::sync::mpsc::sync_channel(1);
    let port = runtime.publication_port();
    let publisher = std::thread::spawn(move || {
        let outcome = port.compare_and_publish(candidate);
        publisher_done_signal.store(true, Ordering::Release);
        publication_finished
            .send(())
            .expect("reader-race completion receiver lives");
        outcome
    });
    let wait_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while publication_gate.wait_count() == 0 && std::time::Instant::now() < wait_deadline {
        std::thread::yield_now();
    }
    assert_eq!(publication_gate.wait_count(), 1);
    let observed_identity = identity.clone();
    let reader = std::thread::spawn(move || {
        let mut observations = Vec::new();
        let completion_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while !reader_done.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        while !publisher_done.load(Ordering::Acquire) {
            assert!(
                std::time::Instant::now() < completion_deadline,
                "reader-race publication must complete within one second"
            );
            let (descriptor, basis) = runtime
                .observe_branch(&observed_identity)
                .expect("concurrent public observation remains complete");
            assert_public_observation_is_complete(&descriptor, &basis);
            let entity_count = runtime
                .read_truth()
                .read_observation(&basis.observation())
                .expect("concurrent complete basis reads")
                .entities()
                .len();
            observations.push((descriptor, entity_count));
            iteration_counter.fetch_add(1, Ordering::Release);
            std::thread::yield_now();
        }
        let (descriptor, basis) = runtime
            .observe_branch(&observed_identity)
            .expect("final public observation remains complete");
        assert_public_observation_is_complete(&descriptor, &basis);
        let entity_count = runtime
            .read_truth()
            .read_observation(&basis.observation())
            .expect("final complete basis reads")
            .entities()
            .len();
        observations.push((descriptor, entity_count));
        (runtime, observations)
    });
    reader_started.store(true, Ordering::Release);
    let reader_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while reader_iterations.load(Ordering::Acquire) == 0
        && std::time::Instant::now() < reader_deadline
    {
        std::thread::yield_now();
    }
    assert!(reader_iterations.load(Ordering::Acquire) > 0);
    drop(held_gate);
    publication_completion
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("reader-race publication completes within one second");
    let performed = match publisher.join().expect("publisher joins") {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("reader-race candidate performs: {outcome:?}"),
    };
    let (runtime, observations) = reader.join().expect("public reader joins");
    let (new_descriptor, new_observation) = runtime
        .observe_branch(&identity)
        .expect("public observation sees the complete next root");
    assert_eq!(
        new_descriptor.root_identity(),
        performed.next_basis().descriptor().root_identity()
    );
    assert_public_observation_is_complete(&new_descriptor, &new_observation);
    assert_ne!(old_descriptor, new_descriptor);
    for (descriptor, entity_count) in observations {
        if descriptor == old_descriptor {
            assert_eq!(entity_count, 1);
        } else if descriptor == new_descriptor {
            assert_eq!(entity_count, 2);
        } else {
            panic!("public reader observed a mixed cutover descriptor: {descriptor:?}");
        }
    }
}

#[test]
fn fork_and_publication_consume_complete_old_and_new_roots() {
    let runtime = runtime_with_test_schema();
    let old_commit = create_entity_outcome(&runtime, "fork-race-anchor");
    let old_basis = crate::tests::support::test_owner_main_basis(&runtime).expect("old basis");
    let old_root_id = old_basis.descriptor().root_identity();
    let (_, fork_source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("old root issues a fork source");

    let mut transaction = runtime
        .begin_branch_transaction(
            &old_basis,
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("fork-race transaction binds");
    transaction
        .push_batch(
            batch_create("fork-race-new-root"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("fork-race candidate prepares");
    let publication_cell = candidate.publication_cell_for_test();
    let publication_gate = Arc::clone(publication_cell.coordination());
    let held_gate = publication_gate.enter();
    let publisher_done = Arc::new(AtomicBool::new(false));
    let publisher_done_signal = Arc::clone(&publisher_done);
    let (publication_finished, publication_completion) = std::sync::mpsc::sync_channel(1);
    let port = runtime.publication_port();
    let publisher = std::thread::spawn(move || {
        let outcome = port.compare_and_publish(candidate);
        publisher_done_signal.store(true, Ordering::Release);
        publication_finished
            .send(())
            .expect("fork-race completion receiver lives");
        outcome
    });
    let wait_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while publication_gate.wait_count() == 0 && std::time::Instant::now() < wait_deadline {
        std::thread::yield_now();
    }
    assert_eq!(publication_gate.wait_count(), 1);

    let race = Arc::new(std::sync::Barrier::new(2));
    let fork_race = Arc::clone(&race);
    let forker = std::thread::spawn(move || {
        let mut successful_forks = Vec::new();
        let mut stale_count = 0;
        fork_race.wait();
        match runtime.fork_branch(BranchId("cutover-fork-0".to_owned()), fork_source) {
            Ok(forked) => record_complete_fork(&runtime, &forked, &mut successful_forks),
            Err(crate::branch::RelationalForkDenial::StaleSource) => stale_count += 1,
            Err(denial) => panic!("concurrent fork has a typed cutover result: {denial:?}"),
        }
        let mut ordinal = 1;
        let completion_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while !publisher_done.load(Ordering::Acquire) && ordinal <= 64 {
            let (_, source) = runtime
                .observe_fork_source(&BranchId("main".to_owned()))
                .expect("current source remains forkable during cutover");
            match runtime.fork_branch(BranchId(format!("cutover-fork-{ordinal}")), source) {
                Ok(forked) => record_complete_fork(&runtime, &forked, &mut successful_forks),
                Err(crate::branch::RelationalForkDenial::StaleSource) => stale_count += 1,
                Err(denial) => panic!("concurrent fork has a typed cutover result: {denial:?}"),
            }
            ordinal += 1;
            std::thread::yield_now();
        }
        while !publisher_done.load(Ordering::Acquire)
            && std::time::Instant::now() < completion_deadline
        {
            std::thread::yield_now();
        }
        assert!(
            publisher_done.load(Ordering::Acquire),
            "fork-race publication must complete within one second"
        );
        let (_, final_source) = runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .expect("new source remains forkable after cutover");
        let final_fork = runtime
            .fork_branch(BranchId(format!("cutover-fork-{ordinal}")), final_source)
            .expect("post-cutover fork consumes the complete new root");
        record_complete_fork(&runtime, &final_fork, &mut successful_forks);
        (runtime, successful_forks, stale_count)
    });
    race.wait();
    drop(held_gate);
    publication_completion
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("fork-race publication completes within one second");
    let performed = match publisher.join().expect("publisher joins") {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("source advances to the complete new root: {outcome:?}"),
    };
    let (runtime, successful_forks, _stale_count) = forker.join().expect("fork racer joins");
    let new_main = crate::tests::support::test_owner_main_basis(&runtime).expect("new main basis");
    assert_eq!(
        new_main.descriptor().root_identity(),
        performed.next_basis().descriptor().root_identity()
    );
    assert_ne!(new_main.descriptor().root_identity(), old_root_id);
    let new_root_id = new_main.descriptor().root_identity();
    let new_commit_id = performed.canonical_commit().commit.commit_id;
    for (descriptor, commit_id, entity_count) in successful_forks {
        if descriptor.root_identity() == old_root_id {
            assert_eq!(commit_id, old_commit.commit.commit_id);
            assert_eq!(entity_count, 1);
        } else if descriptor.root_identity() == new_root_id {
            assert_eq!(commit_id, new_commit_id);
            assert_eq!(entity_count, 2);
        } else {
            panic!("fork consumed a mixed cutover root: {descriptor:?}");
        }
    }
}

fn record_complete_fork(
    runtime: &crate::facade::runtime::RelationalRuntime,
    forked: &crate::branch::RelationalForkOutcome,
    records: &mut Vec<(
        crate::branch::RelationalBranchBasisDescriptor,
        CommitId,
        usize,
    )>,
) {
    let basis = runtime
        .admit_branch_basis(forked.target_identity())
        .expect("successful fork basis is admissible");
    assert_public_observation_is_complete(basis.descriptor(), &basis);
    let commit_id = basis
        .observation()
        .commit_id()
        .expect("non-empty fork carries a commit");
    let entity_count = runtime
        .read_truth()
        .read_observation(&basis.observation())
        .expect("successful fork reads its complete root")
        .entities()
        .len();
    records.push((basis.descriptor().clone(), commit_id, entity_count));
}

fn assert_public_observation_is_complete(
    descriptor: &crate::branch::RelationalBranchBasisDescriptor,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
) {
    let observation = basis.observation();
    assert_eq!(descriptor, observation.descriptor());
    assert_eq!(
        descriptor.root_identity(),
        observation.selected_root_identity()
    );
    match observation.canonical_commit() {
        Some(envelope) => {
            assert_eq!(observation.commit_id(), Some(envelope.commit.commit_id));
            assert_eq!(observation.commit_receipt(), Some(&envelope.commit));
            assert_eq!(observation.canonical_patch(), Some(&envelope.patch));
        }
        None => assert!(observation.commit_id().is_none()),
    }
}
