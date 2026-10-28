use std::sync::{mpsc, Arc, Barrier};
use std::time::Duration;
use worth_execution::ExecutionAllocationPolicy as Allocation;

use crate::tests::support::*;

const CLOSE_START_ACK_TIMEOUT: Duration = Duration::from_secs(1);

#[test]
fn phase2_ports_are_cloneable_sendable_and_shared_borrow_services() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    fn assert_clone_send_sync<T: Clone + Send + Sync>() {}

    assert_clone_send_sync::<crate::facade::mvcc::RelationalPreparationPort>();
    assert_clone_send_sync::<crate::facade::branch::RelationalForkPort>();

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase2-port-anchor");
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase2-preparation-anchor");
    fork_from_main(execution, &runtime, "paused-preparation");
    fork_from_main(execution, &runtime, "progressing-preparation");

    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = crate::mvcc::RelationalOperationControl::uninterrupted()
        .with_boundary_pause_for_test(
            crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation,
            Arc::clone(&reached),
            Arc::clone(&release),
        );
    let mut paused_transaction =
        begin_transaction_with_control(&runtime, "paused-preparation", control);
    paused_transaction
        .push_batch(
            batch_create("paused-preparation-write"),
            Allocation::SystemAllocation,
        )
        .expect("paused transaction stages");
    let mut progressing_transaction = begin_transaction(&runtime, "progressing-preparation");
    progressing_transaction
        .push_batch(
            batch_create("progressing-preparation-write"),
            Allocation::SystemAllocation,
        )
        .expect("progressing transaction stages");

    let paused_cell = runtime
        .history
        .branch_cell(&BranchId("paused-preparation".to_owned()))
        .expect("paused branch cell");
    let paused_contacts_before = paused_cell.coordination().contact_count();
    let paused_waits_before = paused_cell.coordination().wait_count();
    let preparation = runtime.preparation_port();
    let paused_port = preparation.clone();
    let paused_thread = std::thread::spawn(move || {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        paused_port.prepare_branch_transaction(execution, paused_transaction)
    });

    reached.wait();
    assert!(
        !paused_thread.is_finished(),
        "branch A remains paused in preparation"
    );
    let progressing_candidate = preparation
        .prepare_branch_transaction(execution, progressing_transaction)
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase2-fork-anchor");
    fork_from_main(execution, &runtime, "paused-fork-source");
    fork_from_main(execution, &runtime, "progressing-fork-source");
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
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        paused_port.fork_branch_with_test_pause(
            execution,
            BranchId("paused-fork".to_owned()),
            paused_source,
            &paused_reached,
            &paused_release,
        )
    });

    reached.wait();
    assert!(!paused_thread.is_finished(), "branch A fork remains paused");
    port.fork_branch(
        execution,
        BranchId("progressing-fork".to_owned()),
        progressing_source,
    )
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "post-install-close-anchor");
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
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        worker_port.fork_branch_with_post_install_test_pause(
            execution,
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
fn cloned_preparation_ports_share_symbol_identity_and_public_snapshot_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase2-symbol-sharing-anchor");
    fork_from_main(execution, &runtime, "symbol-sharing-a");
    fork_from_main(execution, &runtime, "symbol-sharing-b");
    let symbols_before = runtime.config().identity.symbol_table;
    let preparation = runtime.preparation_port();
    let cloned_preparation = preparation.clone();

    let mut first = begin_transaction(&runtime, "symbol-sharing-a");
    first
        .push_batch(
            batch_create("clone-shared-symbol-a"),
            Allocation::SystemAllocation,
        )
        .expect("first symbol transaction stages");
    let mut second = begin_transaction(&runtime, "symbol-sharing-b");
    second
        .push_batch(
            batch_create("clone-shared-symbol-b"),
            Allocation::SystemAllocation,
        )
        .expect("second symbol transaction stages");

    let first_candidate = cloned_preparation
        .prepare_branch_transaction(execution, first)
        .expect("port clone prepares the first symbol");
    let second_candidate = preparation
        .prepare_branch_transaction(execution, second)
        .expect("original port prepares the second symbol");
    assert_eq!(runtime.config().identity.symbol_table, symbols_before);
    let first = runtime
        .publish_prepared_candidate(execution, first_candidate)
        .unwrap();
    release_test_commit_snapshot(&runtime, &first);
    let symbols = runtime.config().identity.symbol_table;
    let first_symbol = symbols
        .entries
        .iter()
        .find_map(|(symbol, value)| (value == "clone-shared-symbol-a").then_some(*symbol))
        .expect("published clone-admitted symbol is visible in the runtime snapshot");
    assert!(!symbols
        .entries
        .iter()
        .any(|(_, value)| value == "clone-shared-symbol-b"));
    let second = runtime
        .publish_prepared_candidate(execution, second_candidate)
        .unwrap();
    release_test_commit_snapshot(&runtime, &second);
    let second_symbol = runtime
        .config()
        .identity
        .symbol_table
        .entries
        .iter()
        .find_map(|(symbol, value)| (value == "clone-shared-symbol-b").then_some(*symbol))
        .expect("published original-port symbol is visible in the runtime snapshot");
    assert_ne!(
        first_symbol, second_symbol,
        "distinct strings cannot collide across preparation-port clones"
    );

    let symbols_before_discard = runtime.config().identity.symbol_table;
    let mut abandoned = begin_transaction(&runtime, "symbol-sharing-a");
    abandoned
        .push_batch(
            batch_create("clone-abandoned-symbol"),
            Allocation::SystemAllocation,
        )
        .unwrap();
    let abandoned = cloned_preparation
        .prepare_branch_transaction(execution, abandoned)
        .unwrap();
    preparation
        .discard_prepared_candidate(abandoned)
        .expect("candidate discards through the shared owner");
    assert_eq!(
        runtime.config().identity.symbol_table,
        symbols_before_discard
    );
}

fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    target: &str,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(execution, BranchId(target.to_owned()), source)
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

#[path = "owner_service_phase2/closed_owner.rs"]
mod closed_owner;
#[path = "owner_service_phase2/duplicate_fork.rs"]
mod duplicate_fork;
