#[allow(unused_imports)] use crate::relational_certification::ordinary_context::owner_service_completion::owner_lifecycle_observation::*;
pub(crate) use std::sync::mpsc::sync_channel;

pub(crate) use crate::relational_certification::ordinary_context::owner_service_completion::world::empty_runtime;

#[test]
fn descriptive_owner_lifecycle_observes_open_closing_and_closed_without_authority() {
    let host_request = std::sync::Arc::new(
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
    );
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    let lifecycle_port = services.lifecycle_port();
    assert_eq!(
        lifecycle_port.owner_lifecycle_observation(),
        worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Open
    );

    let (reached, release) = (
        std::sync::Arc::new(std::sync::Barrier::new(2)),
        std::sync::Arc::new(std::sync::Barrier::new(2)),
    );
    let control = worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted()
        .with_critical_section_pause_for_test(
            std::sync::Arc::clone(&reached),
            std::sync::Arc::clone(&release),
        );
    let basis = services
        .basis_port()
        .admit_branch_basis(&runtime.main_branch_identity())
        .expect("owner service admits the exact main basis");
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("controlled publication transaction opens");
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new("observe-owner-closing"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("lifecycle observation batch stays in budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(execution, transaction)
        .expect("controlled candidate prepares canonically");

    let (publication_tx, publication_rx) = sync_channel(1);
    let publication_port = services.publication_port();
    std::thread::spawn({
        let publication_request = std::sync::Arc::clone(&host_request);
        move || {
            let execution = &worth_execution::ExecutionRequest::serial(&publication_request);

            let _ = publication_tx.send(publication_port.compare_and_publish(execution, candidate));
        }
    });

    let (owner_close_start_tx, owner_close_start_rx) = sync_channel(1);
    let (owner_dropped_tx, owner_dropped_rx) = sync_channel(1);
    std::thread::spawn(move || {
        let _ = owner_close_start_rx.recv_timeout(OWNER_CLOSE_START_FAIL_SAFE);
        drop(runtime);
        let _ = owner_dropped_tx.send(());
    });
    assert_pause_reached(
        std::sync::Arc::clone(&reached),
        std::sync::Arc::clone(&release),
    );
    let owner_close_started = owner_close_start_tx.send(()).is_ok();
    let closing_reached = wait_until(LIFECYCLE_BOUND, || {
        lifecycle_port.owner_lifecycle_observation()
            == worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Closing
    });
    let owner_waited_for_operation =
        owner_dropped_rx.try_recv() == Err(std::sync::mpsc::TryRecvError::Empty);

    let pause_released = release_controlled_pause(release);
    let publication = publication_rx.recv_timeout(LIFECYCLE_BOUND);
    let publication_performed = matches!(
        &publication,
        Ok(worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(_))
    );
    drop(publication);
    let owner_dropped = owner_dropped_rx.recv_timeout(LIFECYCLE_BOUND);

    assert!(
        owner_close_started,
        "owner close controller disappeared before release"
    );
    assert!(
        closing_reached,
        "owner never exposed Closing within the bound"
    );
    assert!(
        owner_waited_for_operation,
        "owner drop completed while admitted publication remained parked"
    );
    assert_eq!(
        pause_released,
        Ok(()),
        "publication pause release must finish within the bound"
    );
    assert!(
        publication_performed,
        "parked admitted publication must complete after release"
    );
    assert_eq!(owner_dropped, Ok(()), "owner close must drain in bound");
    assert_eq!(
        lifecycle_port.owner_lifecycle_observation(),
        worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Closed
    );
}
