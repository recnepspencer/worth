use std::sync::{mpsc, Arc, Barrier};
use std::time::Duration;

use crate::tests::support::*;

const CLOSE_START_ACK_TIMEOUT: Duration = Duration::from_secs(1);

#[test]
fn phase2_ports_are_cloneable_sendable_and_shared_borrow_services() {
    fn assert_clone_send_sync<T: Clone + Send + Sync>() {}

    assert_clone_send_sync::<crate::facade::mvcc::RelationalPreparationPort>();
    assert_clone_send_sync::<crate::facade::branch::RelationalForkPort>();

    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "phase2-port-anchor");
    let preparation = runtime.preparation_port();
    let forking = runtime.fork_port();
    let _preparation_clone = preparation.clone();
    let _fork_clone = forking.clone();

    let runtime_shared = &runtime;
    let _second_preparation = runtime_shared.preparation_port();
    let _second_fork = runtime_shared.fork_port();
}

#[test]
fn paused_branch_preparation_does_not_block_unrelated_branch_preparation() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "phase2-preparation-anchor");
    fork_from_main(&runtime, "paused-preparation");
    fork_from_main(&runtime, "progressing-preparation");

    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = crate::mvcc::RelationalOperationControl::uninterrupted().with_boundary_pause(
        crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation,
        Arc::clone(&reached),
        Arc::clone(&release),
    );
    let mut paused_transaction =
        begin_transaction_with_control(&runtime, "paused-preparation", control);
    paused_transaction
        .push_batch(batch_create("paused-preparation-write"))
        .expect("paused transaction stages");
    let mut progressing_transaction = begin_transaction(&runtime, "progressing-preparation");
    progressing_transaction
        .push_batch(batch_create("progressing-preparation-write"))
        .expect("progressing transaction stages");

    let paused_cell = runtime
        .history
        .branch_cell(&BranchId("paused-preparation".to_owned()))
        .expect("paused branch cell");
    let paused_contacts_before = paused_cell.coordination().contact_count();
    let paused_waits_before = paused_cell.coordination().wait_count();
    let preparation = runtime.preparation_port();
    let paused_port = preparation.clone();
    let paused_thread =
        std::thread::spawn(move || paused_port.prepare_branch_transaction(paused_transaction));

    reached.wait();
    assert!(
        !paused_thread.is_finished(),
        "branch A remains paused in preparation"
    );
    let progressing_candidate = preparation
        .prepare_branch_transaction(progressing_transaction)
        .expect("branch B prepares while branch A is paused");
    assert!(
        !paused_thread.is_finished(),
        "branch B did not release branch A"
    );
    assert_eq!(
        paused_cell.coordination().contact_count(),
        paused_contacts_before,
        "branch B preparation never contacts branch A coordination",
    );
    assert_eq!(
        paused_cell.coordination().wait_count(),
        paused_waits_before,
        "branch B preparation never waits on branch A coordination",
    );

    release.wait();
    let paused_candidate = paused_thread
        .join()
        .expect("paused preparation worker joins")
        .expect("branch A completes after release");
    preparation
        .discard_prepared_candidate(progressing_candidate)
        .expect("branch B candidate discards through the port");
    preparation
        .discard_prepared_candidate(paused_candidate)
        .expect("branch A candidate discards through the port");
}

#[test]
fn paused_fork_does_not_block_an_unrelated_fork() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "phase2-fork-anchor");
    fork_from_main(&runtime, "paused-fork-source");
    fork_from_main(&runtime, "progressing-fork-source");
    let paused_source_branch = BranchId("paused-fork-source".to_owned());
    let progressing_source_branch = BranchId("progressing-fork-source".to_owned());
    let source_cell = runtime
        .history
        .branch_cell(&paused_source_branch)
        .expect("source branch cell");
    let contacts_before = source_cell.coordination().contact_count();
    let waits_before = source_cell.coordination().wait_count();
    let port = runtime.fork_port();
    let (_, paused_source) = port
        .observe_fork_source(&paused_source_branch)
        .expect("paused fork source");
    let (_, progressing_source) = port
        .observe_fork_source(&progressing_source_branch)
        .expect("progressing fork source");
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let paused_port = port.clone();
    let paused_reached = Arc::clone(&reached);
    let paused_release = Arc::clone(&release);
    let paused_thread = std::thread::spawn(move || {
        paused_port.fork_branch_with_test_pause(
            BranchId("paused-fork".to_owned()),
            paused_source,
            &paused_reached,
            &paused_release,
        )
    });

    reached.wait();
    assert!(!paused_thread.is_finished(), "branch A fork remains paused");
    port.fork_branch(BranchId("progressing-fork".to_owned()), progressing_source)
        .expect("branch B fork completes while branch A is paused");
    assert!(
        !paused_thread.is_finished(),
        "branch B did not release branch A"
    );
    assert_eq!(source_cell.coordination().contact_count(), contacts_before);
    assert_eq!(source_cell.coordination().wait_count(), waits_before);

    release.wait();
    paused_thread
        .join()
        .expect("paused fork worker joins")
        .expect("branch A fork completes after release");
}

#[test]
fn fork_basis_return_holds_owner_admission_across_post_install_close_boundary() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "post-install-close-anchor");
    let port = runtime.fork_port();
    let (_, source) = port
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("fork source observes before the controlled boundary");
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker_port = port.clone();
    let worker_reached = Arc::clone(&reached);
    let worker_release = Arc::clone(&release);
    let worker = std::thread::spawn(move || {
        worker_port.fork_branch_with_post_install_test_pause(
            BranchId("post-install-close-target".to_owned()),
            source,
            &worker_reached,
            &worker_release,
        )
    });

    reached.wait();
    let (close_started_tx, close_started_rx) = mpsc::channel();
    runtime
        .owner_binding()
        .install_test_close_start_ack(close_started_tx);
    let closer = std::thread::spawn(move || {
        drop(runtime);
    });
    close_started_rx
        .recv_timeout(CLOSE_START_ACK_TIMEOUT)
        .expect("close acknowledges lifecycle admission before the fork is released");
    release.wait();

    let (outcome, basis) = worker
        .join()
        .expect("fork worker does not panic")
        .expect("the admitted basis returns across the post-install boundary");
    assert_eq!(basis.identity(), outcome.target_identity());
    closer.join().expect("owner close worker does not panic");
}

#[test]
fn concurrent_duplicate_fork_has_one_winner_and_no_second_target() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "phase2-duplicate-fork-anchor");
    let source_branch = BranchId("main".to_owned());
    let port = runtime.fork_port();
    let (_, first_source) = port
        .observe_fork_source(&source_branch)
        .expect("first exact source");
    let (_, second_source) = port
        .observe_fork_source(&source_branch)
        .expect("second exact source");
    let branch_count_before = runtime.history().branch_cells_snapshot().len();
    let start = Arc::new(Barrier::new(3));

    let first_port = port.clone();
    let first_start = Arc::clone(&start);
    let first = std::thread::spawn(move || {
        first_start.wait();
        first_port.fork_branch(BranchId("duplicate-target".to_owned()), first_source)
    });
    let second_port = port.clone();
    let second_start = Arc::clone(&start);
    let second = std::thread::spawn(move || {
        second_start.wait();
        second_port.fork_branch(BranchId("duplicate-target".to_owned()), second_source)
    });
    start.wait();

    let outcomes = [
        first.join().expect("first fork worker joins"),
        second.join().expect("second fork worker joins"),
    ];
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(
                outcome,
                Err(crate::branch::RelationalForkDenial::DuplicateTarget)
            ))
            .count(),
        1,
    );
    assert_eq!(
        runtime.history().branch_cells_snapshot().len(),
        branch_count_before + 1,
        "the losing reservation leaves no second branch cell",
    );
    let source_root = runtime
        .history
        .branch_cell(&source_branch)
        .and_then(|cell| cell.root())
        .expect("source root");
    let target_root = runtime
        .history
        .branch_cell(&BranchId("duplicate-target".to_owned()))
        .and_then(|cell| cell.root())
        .expect("winning target root");
    assert!(Arc::ptr_eq(&source_root, &target_root));
}

#[test]
fn cloned_preparation_ports_share_symbol_identity_and_public_snapshot_truth() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "phase2-symbol-sharing-anchor");
    fork_from_main(&runtime, "symbol-sharing-a");
    fork_from_main(&runtime, "symbol-sharing-b");
    let preparation = runtime.preparation_port();
    let cloned_preparation = preparation.clone();

    let mut first = begin_transaction(&runtime, "symbol-sharing-a");
    first
        .push_batch(batch_create("clone-shared-symbol-a"))
        .expect("first symbol transaction stages");
    let mut second = begin_transaction(&runtime, "symbol-sharing-b");
    second
        .push_batch(batch_create("clone-shared-symbol-b"))
        .expect("second symbol transaction stages");

    let first_candidate = cloned_preparation
        .prepare_branch_transaction(first)
        .expect("port clone prepares the first symbol");
    let second_candidate = preparation
        .prepare_branch_transaction(second)
        .expect("original port prepares the second symbol");
    let symbols = runtime.config().identity.symbol_table;
    let first_symbol = symbols
        .entries
        .iter()
        .find_map(|(symbol, value)| (value == "clone-shared-symbol-a").then_some(*symbol))
        .expect("clone-admitted symbol is visible in the runtime snapshot");
    let second_symbol = symbols
        .entries
        .iter()
        .find_map(|(symbol, value)| (value == "clone-shared-symbol-b").then_some(*symbol))
        .expect("original-port symbol is visible in the runtime snapshot");
    assert_ne!(
        first_symbol, second_symbol,
        "distinct strings cannot collide across preparation-port clones"
    );

    preparation
        .discard_prepared_candidate(first_candidate)
        .expect("first candidate discards through the shared owner");
    preparation
        .discard_prepared_candidate(second_candidate)
        .expect("second candidate discards through the shared owner");
}

#[test]
fn phase2_ports_deny_after_their_runtime_owner_closes() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "phase2-owner-close-anchor");
    let mut transaction = begin_transaction(&runtime, "main");
    transaction
        .push_batch(batch_create("phase2-owner-close-write"))
        .expect("owner-close transaction stages");
    let preparation = runtime.preparation_port();
    let forking = runtime.fork_port();
    let (_, source) = forking
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("source observes before owner close");
    let runtime_instance_id = runtime.runtime_instance_id();
    drop(runtime);

    assert!(matches!(
        preparation.prepare_branch_transaction(transaction),
        Err(crate::transactions::data::TransactionCommitError::PublicationDenied {
            denial: crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                runtime_instance_id: observed,
            },
            ..
        }) if observed == runtime_instance_id
    ));
    assert!(matches!(
        forking.fork_branch(BranchId("owner-closed-target".to_owned()), source),
        Err(crate::branch::RelationalForkDenial::OwnerUnavailable)
    ));
}

fn fork_from_main(runtime: &crate::runtime::RelationalRuntime, target: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(target.to_owned()), source)
        .expect("test branch fork succeeds");
}

fn begin_transaction(
    runtime: &crate::runtime::RelationalRuntime,
    branch: &str,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("test branch identity");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("test branch basis");
    runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .expect("test branch transaction")
}

fn begin_transaction_with_control(
    runtime: &crate::runtime::RelationalRuntime,
    branch: &str,
    control: crate::mvcc::RelationalOperationControl,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("test branch identity");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("test branch basis");
    runtime
        .begin_branch_transaction_with_control(
            &basis,
            crate::mvcc::RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("controlled branch transaction")
}
