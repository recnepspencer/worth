use std::sync::{Arc, Barrier};

use super::*;
use crate::lifecycle::RuntimeWorldBranchService;

#[test]
fn retirement_tombstones_a_cell_already_held_by_a_ready_publisher() {
    let (fixture, owner, expected) = setup();
    let cell = owner
        .state
        .branches
        .root_cell()
        .expect("the publisher resolves the live product cell");
    let ready = ready_relational(&fixture, &owner, &expected, "retirement-race");
    let history_before = owner.state.history.len();
    let gate = Arc::new(Barrier::new(2));

    let (report, outcome) = std::thread::scope(|scope| {
        let publisher_gate = Arc::clone(&gate);
        let publisher_cell = cell.clone();
        let publisher = scope.spawn(move || {
            publisher_gate.wait();
            ready.publish(
                &publisher_cell,
                CompositeLateCancellationPosture::NotRequested,
            )
        });

        let report = RuntimeWorldBranchService::retire_product_branch(&*owner, &expected)
            .expect("retirement wins before the parked publisher resumes");
        assert_eq!(report.retired_head(), expected.selected_commit());
        gate.wait();
        (
            report,
            publisher
                .join()
                .expect("the publisher returns a typed loss"),
        )
    });

    let RuntimeWorldPublicationOutcome::ProductUnpublished(retained) = outcome else {
        panic!("a publisher holding a retired cell cannot perform")
    };
    assert_eq!(retained.cause(), ProductUnpublishedCause::StaleProductHead);
    assert_eq!(retained.last_observed_head(), Some(expected.snapshot()));
    let costs = retained
        .product_comparison_costs()
        .expect("the product comparison records exact work");
    assert_eq!(costs.cas_attempts(), 1);
    assert_eq!(costs.cas_losses(), 1);
    assert_eq!(costs.history_slots_installed(), 0);
    assert_eq!(owner.state.history.len(), history_before);
    assert_eq!(
        cell.atomic_snapshot().selected_commit(),
        report.retired_head()
    );
    assert!(matches!(
        cell.observe(&owner.state.history, &owner.state.retention),
        Err(crate::branch::ProductBranchReferenceObservationFailure::Retired)
    ));

    let handle = retained.recovery_handle();
    drop(retained);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
}
