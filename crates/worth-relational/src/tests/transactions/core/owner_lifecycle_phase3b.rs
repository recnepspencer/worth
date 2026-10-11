use std::sync::mpsc::{sync_channel, RecvTimeoutError};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use crate::publication::data::DeferredPublicationSettlementError;
use crate::tests::support::*;

/// Every wait in this module is bounded. A court that is red because a thread
/// deadlocked must fail, not hang the suite, so the settlement worker is
/// observed through a bounded completion event before any join.
const COMPLETION_BUDGET: Duration = Duration::from_secs(5);
const NEGATIVE_BUDGET: Duration = Duration::from_millis(100);

/// An owner released while one of its own settlements is running on another
/// thread finishes, and that settlement finishes with a real receipt.
///
/// A bounded join follows completion or disconnection. If a settlement could
/// close the runtime it borrowed, the worker would block on its own admission
/// and this court would fail on its bounded receive rather than hang.
#[test]
fn phase3b_owner_drop_during_off_thread_settlement_completes() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "phase3b-detached-anchor");
    let (reached, release, control) = settlement_pause();
    let performed = perform_write_with_control(&runtime, "main", "phase3b-detached-write", control);
    let commit_id = performed.canonical_commit().commit.commit_id;
    let port = runtime.settlement_port();
    let runtime_instance_id = runtime.runtime_instance_id();
    let admission_observer = runtime.owner_binding();

    let (settled_sender, settled_receiver) = sync_channel(1);
    let worker_port = port.clone();
    let settling = std::thread::spawn(move || {
        let outcome = worker_port
            .settle_performed_publication(performed)
            .map(|result| result.commit.commit_id);
        let _ = settled_sender.send(());
        outcome
    });
    reached.wait();

    let (closed_sender, closed_receiver) = sync_channel(1);
    let owner_drop = std::thread::spawn(move || {
        drop(runtime);
        let _ = closed_sender.send(());
    });
    wait_until_admission_closes(&admission_observer);
    assert_eq!(
        closed_receiver.recv_timeout(NEGATIVE_BUDGET),
        Err(RecvTimeoutError::Timeout),
        "the owner waits for the settlement it already admitted",
    );

    release.wait();
    assert_eq!(
        join_completed_worker(
            settled_receiver.recv_timeout(COMPLETION_BUDGET),
            settling,
            COMPLETION_BUDGET,
            "an admitted settlement completes even though its owner is closing"
        )
        .expect("the admitted settlement produces a real receipt"),
        commit_id,
    );
    join_completed_worker(
        closed_receiver.recv_timeout(COMPLETION_BUDGET),
        owner_drop,
        COMPLETION_BUDGET,
        "the owner finishes once its last admitted operation returns",
    );

    assert!(!port.retains_pending_settlement(commit_id));
    assert!(matches!(
        port.repair_pending_publication_settlement(commit_id),
        Err(DeferredPublicationSettlementError::OwnerUnavailable {
            runtime_instance_id: observed,
        }) if observed == runtime_instance_id
    ));
}

/// While an owner is closing, work it had not yet admitted is denied with the
/// typed owner-unavailable posture, and work it already admitted still runs to
/// a real result.
#[test]
fn phase3b_settlement_admitted_after_owner_drop_is_denied() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "phase3b-close-window-anchor");
    fork_from_main(&runtime, "phase3b-paused");
    fork_from_main(&runtime, "phase3b-late");
    let (reached, release, control) = settlement_pause();
    let paused =
        perform_write_with_control(&runtime, "phase3b-paused", "phase3b-paused-write", control);
    let paused_commit_id = paused.canonical_commit().commit.commit_id;
    let late = perform_write(&runtime, "phase3b-late", "phase3b-late-write");
    let late_commit_id = late.canonical_commit().commit.commit_id;
    let port = runtime.settlement_port();
    let runtime_instance_id = runtime.runtime_instance_id();
    let admission_observer = runtime.owner_binding();

    let (settled_sender, settled_receiver) = sync_channel(1);
    let worker_port = port.clone();
    let settling = std::thread::spawn(move || {
        let outcome = worker_port
            .settle_performed_publication(paused)
            .map(|result| result.commit.commit_id);
        let _ = settled_sender.send(());
        outcome
    });
    reached.wait();

    let owner_drop = std::thread::spawn(move || drop(runtime));
    wait_until_admission_closes(&admission_observer);

    assert!(
        matches!(
            port.settle_performed_publication(late),
            Err(crate::transactions::data::TransactionCommitError::PublicationDenied {
                denial: crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                    runtime_instance_id: observed,
                },
                ..
            }) if observed == runtime_instance_id
        ),
        "a settlement not yet admitted when the owner began closing is denied",
    );
    assert!(matches!(
        port.repair_pending_publication_settlement(late_commit_id),
        Err(DeferredPublicationSettlementError::OwnerUnavailable { .. })
    ));

    release.wait();
    assert_eq!(
        join_completed_worker(
            settled_receiver.recv_timeout(COMPLETION_BUDGET),
            settling,
            COMPLETION_BUDGET,
            "the admitted settlement is not denied by the close it overlapped"
        )
        .expect("the admitted settlement produces a real receipt"),
        paused_commit_id,
    );
    owner_drop
        .join()
        .unwrap_or_else(|worker_panic| std::panic::resume_unwind(worker_panic));
}

/// Recovery replaces a whole runtime. The replaced runtime is closed by that
/// replacement, so a service still bound to it denies against its exact
/// instance rather than addressing the runtime that took its place.
#[test]
fn phase3b_recovery_replacement_closes_the_prior_runtimes_ports() {
    let mut runtime = persisted_runtime_with_test_schema();
    let seeded = create_entity_outcome(&runtime, "phase3b-recovery-anchor");
    let seeded_commit_id = seeded.commit.commit_id;
    release_test_commit_snapshot(&runtime, &seeded);
    let port = runtime.settlement_port();
    let replaced_instance_id = runtime.runtime_instance_id();
    assert_eq!(port.runtime_instance_id(), replaced_instance_id);

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    runtime
        .durability_recovery()
        .recover(plan)
        .expect("the persisted runtime recovers from its own plan");

    assert_ne!(
        runtime.runtime_instance_id(),
        replaced_instance_id,
        "recovery installs a different runtime instance",
    );
    assert!(matches!(
        port.repair_pending_publication_settlement(seeded_commit_id),
        Err(DeferredPublicationSettlementError::OwnerUnavailable {
            runtime_instance_id: observed,
        }) if observed == replaced_instance_id
    ));
}

/// The premises this batch removed stay removed. Each is named where it lived,
/// so reintroducing one fails here instead of quietly restoring the hazard.
#[test]
fn phase3b_removed_lifecycle_premises_stay_removed() {
    const RUNTIME_STATE: &str = include_str!("../../../runtime/state/runtime_state/mod.rs");
    const SETTLEMENT_PORT: &str = include_str!("../../../publication/authority/settlement_port.rs");
    const PREPARATION_RUNTIME: &str =
        include_str!("../../../runtime/state/runtime_state/preparation_runtime.rs");

    assert!(
        !RUNTIME_STATE.contains("DerefMut"),
        "exclusive access to shared runtime state is not proved by an operator",
    );
    assert!(
        !RUNTIME_STATE.contains("fn from_shared"),
        "a handle is minted from a tenure, never from a bare shared pointer",
    );
    assert!(
        !RUNTIME_STATE.contains("impl Drop for RelationalRuntimeState"),
        "closing is owner work, not a consequence of the last pointer being released",
    );
    assert!(
        RUNTIME_STATE.contains("impl Drop for RelationalRuntime {"),
        "the owner handle is what closes, and its own drop runs before its fields",
    );
    assert!(
        !SETTLEMENT_PORT.contains("AdmittedSettlementOwner"),
        "an admitted operation carries a handle whose tenure is its admission",
    );
    assert!(
        !PREPARATION_RUNTIME.contains("RelationalPreparationConfiguration"),
        "there is one configuration authority, not a preparation-side copy of it",
    );
    assert!(
        !PREPARATION_RUNTIME.contains("synchronize_preparation_configuration"),
        "nothing has to be reconciled back into a second configuration",
    );
}

fn settlement_pause() -> (
    Arc<Barrier>,
    Arc<Barrier>,
    crate::mvcc::RelationalOperationControl,
) {
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = crate::mvcc::RelationalOperationControl::uninterrupted().with_boundary_pause(
        crate::mvcc::RelationalInterruptionBoundary::Settlement,
        Arc::clone(&reached),
        Arc::clone(&release),
    );
    (reached, release, control)
}

fn wait_until_admission_closes(observer: &crate::runtime::RelationalRuntimeOwnerBinding) {
    let deadline = Instant::now() + COMPLETION_BUDGET;
    while observer.admit().is_some() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert!(
        observer.admit().is_none(),
        "an owner that has begun closing admits no further operation",
    );
}

fn fork_from_main(runtime: &RelationalRuntime, target: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(target.to_owned()), source)
        .expect("test branch fork succeeds");
}

fn perform_write(
    runtime: &RelationalRuntime,
    branch: &str,
    name: &str,
) -> crate::mvcc::PerformedRelationalCommit {
    let mut transaction =
        test_owner_begin_transaction_for_branch(runtime, BranchId(branch.to_owned()));
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    perform_prepared(runtime, transaction)
}

fn perform_write_with_control(
    runtime: &RelationalRuntime,
    branch: &str,
    name: &str,
    control: crate::mvcc::RelationalOperationControl,
) -> crate::mvcc::PerformedRelationalCommit {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("test branch identity");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("test branch remains owner-admissible");
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            crate::mvcc::RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("controlled branch transaction");
    transaction
        .push_batch(
            batch_create(name),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    perform_prepared(runtime, transaction)
}

fn perform_prepared(
    runtime: &RelationalRuntime,
    transaction: crate::mvcc::BranchBoundRelationalTransaction,
) -> crate::mvcc::PerformedRelationalCommit {
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("candidate prepares");
    match runtime.publication_port().compare_and_publish(candidate) {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("an uncontended candidate performs: {outcome:?}"),
    }
}
