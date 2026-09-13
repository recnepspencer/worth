use crate::retention::component_obligation::ComponentBasisReleaseOutcome;
use crate::retention::registry::RetentionObligationDenial;
use crate::retention::unique_component_pin::ExactComponentPinRequest;
use crate::retention::ComponentBasisDependencyClass;

use super::fixture::{real_fixture, root_commit};

#[test]
fn pair_capacity_denies_before_either_real_component_owner_is_contacted() {
    let mut fixture = real_fixture(1, 4);
    let root = root_commit(&mut fixture);
    let before_relational = fixture.relational_runtime.branch_basis_cost_counters();
    let before_signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");

    let denial = fixture
        .owner
        .issue_observation(&root)
        .expect_err("two vacant exact component keys exceed one unique slot");
    assert_eq!(
        denial,
        RetentionObligationDenial::UniquePinCapacityExhausted {
            maximum_unique_component_pins: 1,
        }
    );
    assert_eq!(fixture.owner.unique_pin_count(), 0);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.in_flight_acquisition_count(), 0);
    let costs = fixture.owner.cost_snapshot();
    assert_eq!(costs.batch_denied(), 1);
    assert_eq!(costs.flights_started(), 0);
    assert_eq!(costs.owner_acquisition_contacts(), 0);
    assert_eq!(costs.relational_contacts(), 0);
    assert_eq!(costs.signal_contacts(), 0);
    assert_eq!(costs.rollbacks(), 0);
    assert_eq!(
        fixture
            .relational_runtime
            .branch_basis_cost_counters()
            .external_retention_acquires,
        before_relational.external_retention_acquires
    );
    assert_eq!(
        fixture
            .signal_port
            .owner_service_cost_snapshot()
            .expect("real Signal owner remains available")
            .retention_registry_contacts(),
        before_signal.retention_registry_contacts()
    );
}

#[test]
fn pair_reservation_capacity_is_checked_for_both_flights_before_contact() {
    let mut fixture = real_fixture(2, 1);
    let root = root_commit(&mut fixture);
    let before_relational = fixture.relational_runtime.branch_basis_cost_counters();
    let before_signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");

    let denial = fixture
        .owner
        .issue_observation(&root)
        .expect_err("the fixed two-key batch exceeds one in-flight reservation");
    assert_eq!(
        denial,
        RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
            maximum_in_flight_reservations: 1,
        }
    );
    assert_eq!(fixture.owner.unique_pin_count(), 0);
    assert_eq!(fixture.owner.in_flight_acquisition_count(), 0);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    let costs = fixture.owner.cost_snapshot();
    assert_eq!(costs.batch_denied(), 1);
    assert_eq!(costs.flights_started(), 0);
    assert_eq!(costs.owner_acquisition_contacts(), 0);
    assert_eq!(costs.rollbacks(), 0);
    assert_eq!(
        fixture
            .relational_runtime
            .branch_basis_cost_counters()
            .external_retention_acquires,
        before_relational.external_retention_acquires
    );
    assert_eq!(
        fixture
            .signal_port
            .owner_service_cost_snapshot()
            .expect("real Signal owner remains available")
            .retention_registry_contacts(),
        before_signal.retention_registry_contacts()
    );
}

#[test]
fn reserved_pair_capacity_drops_without_contact_or_leak() {
    let fixture = real_fixture(2, 2);
    let before = fixture.owner.cost_snapshot();
    let reservation = fixture
        .owner
        .reserve_product_publication_pair()
        .expect("the owner reserves the worst-case pair first");
    assert_eq!(fixture.owner.reserved_unique_pin_capacity(), 2);
    assert_eq!(fixture.owner.reserved_in_flight_acquisition_capacity(), 2);
    assert_eq!(fixture.owner.cost_snapshot(), before);
    drop(reservation);
    assert_eq!(fixture.owner.reserved_unique_pin_capacity(), 0);
    assert_eq!(fixture.owner.reserved_in_flight_acquisition_capacity(), 0);
    assert_eq!(fixture.owner.cost_snapshot(), before);
}

#[test]
fn reserved_pair_preconsumption_denial_returns_live_credit_for_recovery() {
    let fixture = real_fixture(2, 2);
    let foreign = real_fixture(2, 2);
    let before = fixture.owner.cost_snapshot();
    let reservation = fixture
        .owner
        .reserve_product_publication_pair()
        .expect("the local owner reserves one pair");

    let (reservation, denial) = reservation
        .bind_publication(&foreign.basis)
        .expect_err("a foreign composite basis is denied before credit consumption");

    assert!(matches!(
        denial,
        crate::retention::registry::RetentionObligationDenial::ForeignOwner { .. }
    ));
    assert_eq!(fixture.owner.reserved_unique_pin_capacity(), 2);
    assert_eq!(fixture.owner.reserved_in_flight_acquisition_capacity(), 2);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    let after_denial = fixture.owner.cost_snapshot();
    assert_eq!(after_denial.batch_denied(), before.batch_denied() + 1);
    assert_eq!(
        after_denial.owner_acquisition_contacts(),
        before.owner_acquisition_contacts()
    );
    drop(reservation);
    assert_eq!(fixture.owner.reserved_unique_pin_capacity(), 0);
    assert_eq!(fixture.owner.reserved_in_flight_acquisition_capacity(), 0);
}

#[test]
fn reserved_pair_binds_to_owner_issued_publication_and_releases_once() {
    let fixture = real_fixture(4, 4);
    let reservation = fixture
        .owner
        .reserve_product_publication_pair()
        .expect("pair capacity reservation");
    let publication = reservation
        .bind_publication(&fixture.basis)
        .expect("reserved pair binds its exact basis");
    assert_eq!(fixture.owner.reserved_unique_pin_capacity(), 0);
    assert_eq!(fixture.owner.reserved_in_flight_acquisition_capacity(), 0);
    assert_eq!(fixture.owner.active_component_obligation_count(), 2);
    drop(publication);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.unique_pin_count(), 2);
    assert_eq!(fixture.owner.reclaim(2).reclaimed(), 2);
}

#[test]
fn foreign_batch_owner_is_rejected_before_pair_reservation() {
    let fixture = real_fixture(2, 2);
    let mut foreign_fixture = real_fixture(2, 2);
    let foreign_root = root_commit(&mut foreign_fixture);
    let denial = fixture
        .owner
        .issue_observation(&foreign_root)
        .expect_err("the retention registry cannot admit a foreign World owner");
    assert!(matches!(
        denial,
        RetentionObligationDenial::ForeignOwner { .. }
    ));
    assert_eq!(fixture.owner.unique_pin_count(), 0);
    assert_eq!(fixture.owner.in_flight_acquisition_count(), 0);
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        0
    );
}

#[test]
fn ready_plus_vacant_pair_counts_only_the_one_new_exact_slot() {
    let fixture = real_fixture(2, 4);
    let before_relational = fixture.relational_runtime.branch_basis_cost_counters();
    let before_signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");
    let relational = fixture
        .owner
        .issue_component(ExactComponentPinRequest::relational(
            &fixture.basis,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect("first real Relational component claim");
    let before_pair = fixture.owner.cost_snapshot();

    let publication = fixture
        .owner
        .issue_publication(&fixture.basis)
        .expect("ready Relational plus vacant Signal fits the bound");
    assert_eq!(fixture.owner.unique_pin_count(), 2);
    assert_eq!(fixture.owner.active_component_obligation_count(), 3);
    assert_eq!(fixture.owner.in_flight_acquisition_count(), 0);
    let after_pair = fixture.owner.cost_snapshot();
    assert_eq!(
        after_pair.flights_started(),
        before_pair.flights_started() + 1
    );
    assert_eq!(
        after_pair.owner_acquisition_contacts(),
        before_pair.owner_acquisition_contacts() + 1
    );
    assert_eq!(
        after_pair.relational_contacts(),
        before_pair.relational_contacts()
    );
    assert_eq!(
        after_pair.signal_contacts(),
        before_pair.signal_contacts() + 1
    );
    assert_eq!(
        after_pair.unique_pin_hits(),
        before_pair.unique_pin_hits() + 1,
        "only the ready component is a unique-pin hit; the vacancy starts one flight"
    );
    assert_eq!(
        fixture
            .relational_runtime
            .branch_basis_cost_counters()
            .external_retention_acquires,
        before_relational.external_retention_acquires + 1
    );
    assert_eq!(
        fixture
            .signal_port
            .owner_service_cost_snapshot()
            .expect("real Signal owner remains available")
            .retention_registry_contacts(),
        before_signal.retention_registry_contacts() + 1
    );

    drop(publication);
    drop(relational);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.cost_snapshot().owner_drop_releases(), 2);
    assert_eq!(fixture.owner.reclaim(2).reclaimed(), 2);
}

#[test]
fn same_key_reacquisition_gets_a_new_lease_generation_and_one_release_each() {
    let fixture = real_fixture(2, 2);
    let before_relational = fixture.relational_runtime.branch_basis_cost_counters();
    let before_signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");
    let first = fixture
        .owner
        .issue_component(ExactComponentPinRequest::signal(
            &fixture.basis,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect("first exact Signal claim");
    let first_identity = first.lease_identity();
    assert_eq!(
        first
            .try_release()
            .expect("first generation has one owner-backed release")
            .outcome(),
        ComponentBasisReleaseOutcome::OwnerReleased
    );
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);

    let second = fixture
        .owner
        .issue_component(ExactComponentPinRequest::signal(
            &fixture.basis,
            ComponentBasisDependencyClass::HistoricalInspection,
        ))
        .expect("reacquisition reopens the released exact key");
    assert_ne!(second.lease_identity(), first_identity);
    assert_eq!(fixture.owner.unique_pin_count(), 1);
    drop(second);

    let relational = fixture.relational_runtime.branch_basis_cost_counters();
    let signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");
    assert_eq!(fixture.owner.cost_snapshot().owner_release_contacts(), 1);
    assert_eq!(fixture.owner.cost_snapshot().owner_drop_releases(), 1);
    assert_eq!(fixture.owner.cost_snapshot().owner_drop_releases(), 1);
    assert_eq!(
        relational.external_retention_acquires,
        before_relational.external_retention_acquires
    );
    assert_eq!(
        signal.retention_registry_contacts(),
        before_signal.retention_registry_contacts() + 2
    );
    assert_eq!(fixture.owner.reclaim(1).reclaimed(), 1);
}

#[test]
fn repeated_composite_use_has_fixed_owner_contacts_and_bounded_reclamation() {
    let mut fixture = real_fixture(2, 4);
    let root = root_commit(&mut fixture);
    let before_relational = fixture.relational_runtime.branch_basis_cost_counters();
    let before_signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");
    let mut observations = Vec::new();
    for _ in 0..32 {
        observations.push(
            fixture
                .owner
                .issue_observation(&root)
                .expect("each repeated use shares the exact owner lease"),
        );
    }

    let costs = fixture.owner.cost_snapshot();
    assert_eq!(costs.batch_admitted(), 32);
    assert_eq!(costs.batch_denied(), 0);
    assert_eq!(costs.flights_started(), 2);
    assert_eq!(costs.single_flight_joins(), 0);
    assert_eq!(costs.owner_acquisition_contacts(), 2);
    assert_eq!(costs.relational_contacts(), 1);
    assert_eq!(costs.signal_contacts(), 1);
    assert_eq!(costs.unique_pin_hits(), 62);
    assert_eq!(fixture.owner.unique_pin_count(), 2);
    assert_eq!(fixture.owner.active_component_obligation_count(), 64);
    assert_eq!(
        fixture
            .relational_runtime
            .branch_basis_cost_counters()
            .external_retention_acquires,
        before_relational.external_retention_acquires + 1
    );
    assert_eq!(
        fixture
            .signal_port
            .owner_service_cost_snapshot()
            .expect("real Signal owner remains available")
            .retention_registry_contacts(),
        before_signal.retention_registry_contacts() + 1
    );

    drop(observations);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.cost_snapshot().owner_drop_releases(), 2);
    let after_relational = fixture.relational_runtime.branch_basis_cost_counters();
    assert_eq!(
        after_relational.external_retention_releases,
        before_relational.external_retention_releases + 1
    );
    assert_eq!(
        after_relational.retained_basis_registry_entries,
        before_relational.retained_basis_registry_entries
    );
    assert_eq!(fixture.owner.reclaim(2).reclaimed(), 2);
}
