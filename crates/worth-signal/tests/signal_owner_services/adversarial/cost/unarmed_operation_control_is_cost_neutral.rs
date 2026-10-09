//! Unarmed operation control is cost neutral.

use super::*;

#[test]
fn unarmed_operation_control_is_cost_neutral() {
    let controlled = AdversarialWorld::new();
    let ordinary = AdversarialWorld::new();
    controlled
        .runtime
        .as_ref()
        .expect("the controlled root remains live")
        .owner_operation_control()
        .expect("obtaining the unarmed control handle succeeds");
    let controlled_before = controlled
        .basis
        .owner_service_cost_snapshot()
        .expect("the controlled owner is open");
    let ordinary_before = ordinary
        .basis
        .owner_service_cost_snapshot()
        .expect("the ordinary owner is open");
    controlled
        .mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::execution_custody::operational_serial_request(),
            ),
            &controlled.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the controlled operation succeeds");
    ordinary
        .mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::execution_custody::operational_serial_request(),
            ),
            &ordinary.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the ordinary operation succeeds");
    let controlled_after = controlled
        .basis
        .owner_service_cost_snapshot()
        .expect("the controlled owner remains open");
    let ordinary_after = ordinary
        .basis
        .owner_service_cost_snapshot()
        .expect("the ordinary owner remains open");
    assert_eq!(
        controlled_after.canonical_movements() - controlled_before.canonical_movements(),
        ordinary_after.canonical_movements() - ordinary_before.canonical_movements()
    );
    assert_eq!(
        controlled_after.target_cell_contacts() - controlled_before.target_cell_contacts(),
        ordinary_after.target_cell_contacts() - ordinary_before.target_cell_contacts()
    );
    assert_eq!(
        controlled_after.branch_registry_entries_scanned()
            - controlled_before.branch_registry_entries_scanned(),
        ordinary_after.branch_registry_entries_scanned()
            - ordinary_before.branch_registry_entries_scanned()
    );
}
