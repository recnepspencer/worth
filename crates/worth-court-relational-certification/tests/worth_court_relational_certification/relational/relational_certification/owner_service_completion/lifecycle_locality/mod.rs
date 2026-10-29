#[allow(unused_imports)] use crate::relational_certification::ordinary_context::owner_service_completion::lifecycle_locality::*;
pub(crate) use {std::sync::mpsc::sync_channel};
pub(crate) use {std::sync::Arc, std::sync::Barrier};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {crate::relational_certification::ordinary_context::owner_service_completion::world::committed_fork};
pub(crate) use {worth_relational::facade::mvcc::RelationalOperationControl, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent};
pub(crate) use {worth_relational::facade::transactions::WorkerIntentBatch};

#[test]
fn parked_branch_lifecycle_call_does_not_block_unrelated_basis_service() {
    let host_request = std::sync::Arc::new(
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
    );
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, services, parked_identity) = committed_fork(execution, "parked-lifecycle");
    let parked_basis = services
        .basis_port()
        .admit_branch_basis(&parked_identity)
        .expect("the parked branch basis is owner-admitted");
    let reached = std::sync::Arc::new(std::sync::Barrier::new(2));
    let release = std::sync::Arc::new(std::sync::Barrier::new(2));
    let control = worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted()
        .with_critical_section_pause_for_test(std::sync::Arc::clone(&reached), std::sync::Arc::clone(&release));
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &parked_basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("the exact parked basis opens a controlled transaction");
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new("park-lifecycle-coordination"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the locality batch remains within its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(execution, transaction)
        .expect("the controlled transaction prepares canonically");

    let publication_port = services.publication_port();
    let publication_worker = std::thread::spawn({
        let publication_request = std::sync::Arc::clone(&host_request);
        move || {
            let execution = &worth_execution::ExecutionRequest::serial(&publication_request);
            publication_port.compare_and_publish(execution, candidate)
        }
    });
    assert_pause_reached(std::sync::Arc::clone(&reached), std::sync::Arc::clone(&release));

    let waits_before = coordination_waits(&runtime, &parked_identity);
    let lifecycle_port = services.lifecycle_port();
    let lifecycle_identity = parked_identity.clone();
    let lifecycle_worker = std::thread::spawn({
        let lifecycle_request = std::sync::Arc::clone(&host_request);
        move || {
            let execution = &worth_execution::ExecutionRequest::serial(&lifecycle_request);
            lifecycle_port.archive_branch(execution, &lifecycle_identity)
        }
    });
    let lifecycle_is_parked =
        wait_for_coordination_wait(&runtime, &parked_identity, waits_before, LOCALITY_TIMEOUT);

    let main_identity = runtime.main_branch_identity();
    let basis_port = services.basis_port();
    let (completed_tx, completed_rx) = sync_channel(1);
    let basis_worker = std::thread::spawn(move || {
        let result = basis_port
            .observe_branch(&main_identity)
            .map(|(descriptor, _basis)| descriptor.branch_id().clone());
        let _ = completed_tx.send(result);
    });
    let unrelated_result = completed_rx.recv_timeout(LOCALITY_TIMEOUT);
    let lifecycle_stayed_parked = !lifecycle_worker.is_finished();

    release.wait();
    let publication = publication_worker
        .join()
        .expect("the parked publication worker does not panic");
    let archived = lifecycle_worker
        .join()
        .expect("the parked lifecycle worker does not panic");
    basis_worker
        .join()
        .expect("the unrelated basis worker does not panic");

    assert!(
        lifecycle_is_parked,
        "the lifecycle call never contended on its exact branch coordination cell"
    );
    assert!(
        lifecycle_stayed_parked,
        "the branch lifecycle call completed before its target cell was released"
    );
    assert!(
        matches!(unrelated_result, Ok(Ok(ref branch)) if branch.0 == "main"),
        "unrelated basis work must finish while lifecycle remains parked: {unrelated_result:?}"
    );
    archived.expect("archive completes after the exact branch cell is released");
    let performed = match publication {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("parked canonical publication did not perform: {outcome:?}"),
    };
    services
        .settlement_port()
        .settle_performed_publication(performed)
        .expect("the performed locality publication settles canonically");
}
