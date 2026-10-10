use super::fixture::real_fixture;
use crate::inspection::RuntimeWorldRetentionKey;
use crate::retention::registry::RuntimeWorldRetentionOwner;
use crate::retention::{ComponentBasisDependencyClass, ExactComponentPinRequest};

#[test]
fn equal_contents_reclaim_the_declared_component_in_both_insertion_orders() {
    let fixture = real_fixture(4, 4);
    let relational = RuntimeWorldRetentionKey::relational(&fixture.basis);
    let signal = RuntimeWorldRetentionKey::signal(&fixture.basis);
    // Independent hash seeds and reversed insertion must both preserve the
    // declared Relational-before-Signal component order.
    for _ in 0..16 {
        for reversed in [false, true] {
            let owner = RuntimeWorldRetentionOwner::new(
                fixture.owner_identity,
                fixture
                    .relational_runtime
                    .owner_component_services()
                    .basis_port(),
                fixture.signal_port.clone(),
                fixture.budgets.unique_exact_component_pins(),
                fixture.budgets.in_flight_pin_acquisition_reservations(),
                fixture.budgets.active_observations(),
            );
            let requests = [
                ExactComponentPinRequest::relational(
                    &fixture.basis,
                    ComponentBasisDependencyClass::ActivePublicationAttempt,
                ),
                ExactComponentPinRequest::signal(
                    &fixture.basis,
                    ComponentBasisDependencyClass::ActivePublicationAttempt,
                ),
            ];
            let order = if reversed { [1, 0] } else { [0, 1] };
            for index in order {
                drop(owner.issue_component(requests[index]).unwrap());
            }
            let report = owner.reclaim(1);
            assert_eq!(report.examined(), 1);
            assert_eq!(report.reclaimed(), 1);
            assert!(owner.inspect_key(&relational).unwrap().is_none());
            assert!(owner.inspect_key(&signal).unwrap().is_some());
        }
    }
}
