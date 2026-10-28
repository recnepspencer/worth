use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::data::PublicationSettlementRepairError;
use crate::tests::support::*;
use crate::transactions::data::{CommitResult, TransactionCommitError};

type SettlementPort = crate::facade::mvcc::RelationalSettlementPort;
type RepairOutcome = Result<RelationalCommitReceipt, PublicationSettlementRepairError>;

/// The settlement contract is a shared-borrow contract. Coercing each governed
/// operation to a function pointer whose first parameter is a shared reference
/// fails to compile the moment one of them reclaims an exclusive receiver.
#[test]
fn phase3_settlement_receivers_take_a_shared_borrow() {
    let _settle: fn(
        &RelationalRuntime,
        crate::mvcc::PerformedRelationalCommit,
    ) -> Result<CommitResult, TransactionCommitError> =
        RelationalRuntime::settle_performed_publication;
    let _repair_identity: fn(&RelationalRuntime, CommitId) -> RepairOutcome =
        RelationalRuntime::repair_pending_publication_settlement;
    let _commit: fn(
        &RelationalRuntime,
        &worth_execution::ExecutionRequest<'_, '_>,
        crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Result<CommitResult, TransactionCommitError> =
        RelationalRuntime::commit_branch_transaction;

    let _port_settle: fn(
        &SettlementPort,
        crate::mvcc::PerformedRelationalCommit,
    ) -> Result<CommitResult, TransactionCommitError> =
        SettlementPort::settle_performed_publication;
    let _port_repair_identity: fn(&SettlementPort, CommitId) -> RepairOutcome =
        SettlementPort::repair_pending_publication_settlement;
}

/// Owner authority keeps its exclusive receiver and the services a runtime
/// issues keep their shared one. Coercing each to a function pointer fails to
/// compile the moment either side takes the other's receiver.
#[test]
fn phase3b_owner_authority_and_owner_services_keep_their_receivers() {
    let _initial_schema: fn(
        &mut RelationalRuntime,
    ) -> Result<
        crate::runtime::RelationalInitialSchemaInstallation<'_>,
        crate::runtime::RelationalInitialSchemaInstallationDenial,
    > = RelationalRuntime::prepare_initial_schema_installation;
    let _settlement_service: fn(&RelationalRuntime) -> SettlementPort =
        RelationalRuntime::settlement_port;
    let _publication_service: fn(&RelationalRuntime) -> crate::mvcc::RelationalPublicationPort =
        RelationalRuntime::publication_port;
    let _configuration: fn(&RelationalRuntime) -> crate::runtime::RelationalRuntimeConfig =
        RelationalRuntime::config;
}

/// The settlement service is an independently borrowable, cloneable owner
/// service, and the whole convenience commit path runs through a shared borrow.
#[test]
fn phase3_settlement_port_is_a_cloneable_shared_borrow_service() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    fn assert_clone_send_sync<T: Clone + Send + Sync>() {}

    assert_clone_send_sync::<SettlementPort>();

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase3-settlement-anchor");
    let runtime_shared = &runtime;
    let settlement = runtime_shared.settlement_port();
    let cloned = settlement.clone();
    let _second = runtime_shared.settlement_port();
    assert_eq!(
        cloned.runtime_instance_id(),
        runtime_shared.runtime_instance_id(),
    );

    let mut transaction = test_owner_begin_transaction_for_main(runtime_shared);
    transaction
        .push_batch(
            batch_create("phase3-shared-borrow-commit"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let committed = runtime_shared
        .commit_branch_transaction(execution, transaction)
        .expect("the convenience commit path settles through a shared borrow");
    assert!(!cloned.retains_pending_settlement(committed.commit.commit_id));
    release_test_commit_snapshot(&runtime, &committed);
}

/// A settlement service outlives no owner. Once the runtime is gone every
/// governed operation answers the typed owner-unavailable posture instead of
/// addressing freed authority.
#[test]
fn phase3_settlement_port_denies_after_its_runtime_owner_closes() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "phase3-owner-close-anchor");
    let performed = perform_write(execution, &runtime, "main", "phase3-owner-close-write");
    let commit_id = performed.canonical_commit().commit.commit_id;
    let settlement = runtime.settlement_port();
    let runtime_instance_id = runtime.runtime_instance_id();
    assert!(settlement.retains_pending_settlement(commit_id));

    drop(runtime);

    assert!(!settlement.retains_pending_settlement(commit_id));
    assert!(matches!(
        settlement.repair_pending_publication_settlement(commit_id),
        Err(PublicationSettlementRepairError::OwnerUnavailable {
            runtime_instance_id: observed,
        }) if observed == runtime_instance_id
    ));
    assert!(matches!(
        settlement.settle_performed_publication(performed),
        Err(TransactionCommitError::PublicationDenied {
            denial: crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                runtime_instance_id: observed,
            },
            ..
        }) if observed == runtime_instance_id
    ));
}

/// Settlement is owner work a caller may hand to another thread. The witness
/// travels with the service, and the runtime the caller still holds observes
/// the terminal effect.
#[test]
fn phase3_settlement_port_settles_a_performed_publication_from_another_thread() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "phase3-offthread-anchor");
    let performed = perform_write(execution, &runtime, "main", "phase3-offthread-write");
    let commit_id = performed.canonical_commit().commit.commit_id;
    let settlement = runtime.settlement_port();
    let worker_port = settlement.clone();

    let settled = std::thread::spawn(move || worker_port.settle_performed_publication(performed))
        .join()
        .expect("settlement worker joins")
        .expect("a cloned settlement service settles from another thread");

    assert_eq!(settled.commit.commit_id, commit_id);
    assert!(!settlement.retains_pending_settlement(commit_id));
    assert_eq!(
        runtime
            .history()
            .immutable_commit_receipt(commit_id)
            .expect("off-thread settlement is visible to the owner")
            .commit_id,
        commit_id,
    );
    let child = create_entity_outcome(execution, &runtime, "phase3-child-after-offthread");
    assert_eq!(child.commit.parents, vec![commit_id]);
    release_test_commit_snapshot(&runtime, &settled);
    release_test_commit_snapshot(&runtime, &child);
}

/// A performed publication whose witness was lost is settled through the same
/// shared-borrow service by commit identity, without the runtime ever being
/// borrowed exclusively, and is recorded in the canonical log exactly once.
#[test]
fn phase3_settlement_port_repairs_a_lost_witness_by_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "phase3-repair-anchor");
    let settlement = runtime.settlement_port();

    let abandoned = perform_write(execution, &runtime, "main", "phase3-identity-repair-write");
    let identity_commit_id = abandoned.canonical_commit().commit.commit_id;
    drop(abandoned);
    assert!(settlement.retains_pending_settlement(identity_commit_id));
    let recovered = settlement
        .repair_pending_publication_settlement(identity_commit_id)
        .expect("the service recovers work whose witness was lost");
    assert_eq!(recovered.commit_id, identity_commit_id);
    assert!(!settlement.retains_pending_settlement(identity_commit_id));

    let child = create_entity_outcome(execution, &runtime, "phase3-child-after-repair");
    assert_eq!(child.commit.parents, vec![identity_commit_id]);
    assert_eq!(
        runtime
            .durability()
            .durable_log()
            .iter()
            .filter(|entry| entry.envelope().commit.commit_id == identity_commit_id)
            .count(),
        1,
        "the repaired settlement is recorded exactly once",
    );
    release_test_commit_snapshot(&runtime, &child);
}

/// A performed witness carries the source runtime's pending record. A foreign
/// settlement service must reject that record before claiming its executor
/// gate, leaving the source runtime able to recover by commit identity.
#[test]
fn phase3_foreign_settlement_denial_preserves_source_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    create_entity(
        execution,
        &source,
        "phase3-foreign-settlement-source-anchor",
    );
    let foreign = durable_runtime_with_test_schema();
    create_entity(
        execution,
        &foreign,
        "phase3-foreign-settlement-target-anchor",
    );

    let performed = perform_write(
        execution,
        &source,
        "main",
        "phase3-foreign-settlement-write",
    );
    let commit_id = performed.canonical_commit().commit.commit_id;
    let source_durable_before = source.durability().durable_log().len();
    let foreign_durable_before = foreign.durability().durable_log().len();
    let source_settlement = source.settlement_port();
    assert!(source_settlement.retains_pending_settlement(commit_id));

    match foreign
        .settlement_port()
        .settle_performed_publication(performed)
    {
        Err(TransactionCommitError::PublicationDenied {
            denial:
                crate::mvcc::RelationalPublicationDenial::ForeignRuntime {
                    expected_runtime_instance_id,
                    actual_runtime_instance_id,
                },
            ..
        }) => {
            assert_eq!(expected_runtime_instance_id, foreign.runtime_instance_id());
            assert_eq!(actual_runtime_instance_id, source.runtime_instance_id());
        }
        outcome => panic!("foreign settlement must return its typed denial: {outcome:?}"),
    }

    assert_eq!(
        foreign.durability().durable_log().len(),
        foreign_durable_before
    );
    assert!(source_settlement.retains_pending_settlement(commit_id));
    let recovered = source_settlement
        .repair_pending_publication_settlement(commit_id)
        .expect("the source runtime retains recovery after foreign denial");
    assert_eq!(recovered.commit_id, commit_id);
    assert_eq!(
        source.durability().durable_log().len(),
        source_durable_before + 1,
        "the source settlement appends exactly once",
    );
    assert!(!source_settlement.retains_pending_settlement(commit_id));
}

/// Deferred repair diagnostics describe the service being asked as expected
/// and the carrier's source as actual. Rejection must leave the source record
/// and exact carrier usable for its rightful owner.
pub(super) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target: &str,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(execution, BranchId(target.to_owned()), source)
        .expect("test branch fork succeeds");
}

fn perform_write(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    perform_prepared(execution, runtime, transaction)
}

pub(super) fn perform_write_with_control(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    perform_prepared(execution, runtime, transaction)
}

fn perform_prepared(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    transaction: crate::mvcc::BranchBoundRelationalTransaction,
) -> crate::mvcc::PerformedRelationalCommit {
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .expect("candidate prepares");
    match runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("an uncontended candidate performs: {outcome:?}"),
    }
}
