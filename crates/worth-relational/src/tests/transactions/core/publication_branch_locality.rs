use crate::tests::support::*;
use std::sync::Arc;

#[test]
fn paused_storm_publication_does_not_contact_or_wait_on_maintenance() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "branch-local-publication-anchor");
    fork_publication_branch(&mut runtime, "storm");
    fork_publication_branch(&mut runtime, "maintenance");

    let storm = prepare_publication(&mut runtime, "storm", "storm-publication");
    let storm_cell = storm.publication_cell_for_test();
    let maintenance = prepare_publication(&mut runtime, "maintenance", "maintenance-publication");
    let maintenance_cell = maintenance.publication_cell_for_test();
    let position_reservations_before = runtime.patch_position_reservation_counters();

    let main_coordination = Arc::clone(
        runtime
            .history
            .branch_cell(&BranchId("main".to_owned()))
            .expect("main branch remains registered")
            .coordination(),
    );
    let main_contacts_before = main_coordination.contact_count();
    let main_waits_before = main_coordination.wait_count();
    let maintenance_contacts_before = maintenance_cell.coordination().contact_count();
    let maintenance_waits_before = maintenance_cell.coordination().wait_count();
    let storm_gate = Arc::clone(storm_cell.coordination());
    let mut held_storm_gate = Some(storm_gate.enter());
    let storm_contacts_after_test_gate = storm_gate.contact_count();
    let (storm_finished, storm_completion) = std::sync::mpsc::sync_channel(1);
    let storm_port = runtime.publication_port();
    let storm_thread = std::thread::spawn(move || {
        let outcome = storm_port.compare_and_publish(storm);
        storm_finished
            .send(())
            .expect("storm completion receiver lives");
        outcome
    });
    let wait_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while storm_gate.wait_count() == 0 && std::time::Instant::now() < wait_deadline {
        std::thread::yield_now();
    }
    assert_eq!(storm_gate.wait_count(), 1, "storm waits on storm only");
    assert_eq!(
        storm_gate.contact_count(),
        storm_contacts_after_test_gate + 1
    );
    assert!(!storm_thread.is_finished(), "storm remains paused");

    let maintenance_started = std::time::Instant::now();
    let (maintenance_finished, maintenance_completion) = std::sync::mpsc::sync_channel(1);
    let maintenance_port = runtime.publication_port();
    let maintenance_thread = std::thread::spawn(move || {
        let outcome = maintenance_port.compare_and_publish(maintenance);
        maintenance_finished
            .send(())
            .expect("maintenance completion receiver lives");
        outcome
    });
    if maintenance_completion
        .recv_timeout(std::time::Duration::from_secs(1))
        .is_err()
    {
        drop(held_storm_gate.take());
        panic!("maintenance publication blocked behind the held Storm gate");
    }
    let maintenance_performed = match maintenance_thread
        .join()
        .expect("maintenance publisher joins")
    {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("maintenance publishes while storm is paused: {outcome:?}"),
    };
    assert!(maintenance_started.elapsed() < std::time::Duration::from_secs(1));
    assert!(
        !storm_thread.is_finished(),
        "maintenance did not release storm"
    );
    assert_eq!(
        maintenance_cell.coordination().contact_count(),
        maintenance_contacts_before + 1
    );
    assert_eq!(
        maintenance_cell.coordination().wait_count(),
        maintenance_waits_before
    );
    assert_eq!(main_coordination.contact_count(), main_contacts_before);
    assert_eq!(main_coordination.wait_count(), main_waits_before);
    assert_eq!(
        storm_gate.contact_count(),
        storm_contacts_after_test_gate + 1,
        "maintenance publication never contacts Storm coordination"
    );

    drop(held_storm_gate.take());
    storm_completion
        .recv_timeout(std::time::Duration::from_secs(1))
        .expect("storm publication completes within one second after release");
    let storm_performed = match storm_thread.join().expect("storm publisher joins") {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("storm performs after its branch gate opens: {outcome:?}"),
    };
    assert_ne!(
        storm_performed.next_basis().descriptor().root_identity(),
        maintenance_performed
            .next_basis()
            .descriptor()
            .root_identity(),
        "concurrent candidates reserve distinct immutable-root identities"
    );
    let position_reservations_after = runtime.patch_position_reservation_counters();
    assert_eq!(
        position_reservations_after.assignments - position_reservations_before.assignments,
        2
    );
    assert_eq!(
        position_reservations_after.contacts - position_reservations_before.contacts,
        2
    );
    assert_eq!(
        position_reservations_after.deferrals - position_reservations_before.deferrals,
        0,
        "branch-local pause occurs before the bounded atomic reservation"
    );
    assert_eq!(position_reservations_after.overflows, 0);
}

fn prepare_publication(
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    branch: &str,
    entity: &str,
) -> crate::facade::mvcc::PreparedRelationalCommitCandidate {
    let mut transaction = begin_publication_transaction(runtime, branch);
    transaction
        .push_batch(
            batch_create(entity),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("publication candidate prepares")
}

fn fork_publication_branch(runtime: &mut crate::facade::runtime::RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(branch.to_owned()), source)
        .expect("publication branch fork succeeds");
}

fn begin_publication_transaction(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
) -> crate::facade::mvcc::BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("publication branch identity exists");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("publication branch basis is admitted");
    runtime
        .begin_branch_transaction(
            &basis,
            crate::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("publication transaction binds")
}
