use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest;

use crate::history::CompositeRuntimeWorldCommit;
use crate::retention::component_obligation::ObservationRetentionObligation;
use crate::retention::registry::{RetentionObligationDenial, RuntimeWorldRetentionOwner};
use crate::retention::unique_component_pin::ExactComponentPinRequest;
use crate::retention::ComponentClaimKind;

use super::fixture::{real_fixture, root_commit};

type Owner = RuntimeWorldRetentionOwner<(), (), ()>;
type ResultValue = Result<ObservationRetentionObligation, RetentionObligationDenial>;

fn wait_for_joins(owner: &Owner, expected: u64) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while owner.cost_snapshot().single_flight_joins() < expected {
        assert!(
            Instant::now() < deadline,
            "all batch joiners reached the flight"
        );
        thread::yield_now();
    }
}

fn start_claimant(
    owner: Arc<Owner>,
    commit: Arc<CompositeRuntimeWorldCommit>,
) -> JoinHandle<ResultValue> {
    thread::spawn(move || owner.issue_observation(&commit))
}

fn start_joiners(
    owner: Arc<Owner>,
    commit: Arc<CompositeRuntimeWorldCommit>,
    count: usize,
) -> Vec<JoinHandle<ResultValue>> {
    let mut joiners = Vec::new();
    for _ in 0..count {
        let owner = Arc::clone(&owner);
        let commit = Arc::clone(&commit);
        joiners.push(thread::spawn(move || owner.issue_observation(&commit)));
    }
    joiners
}

fn collect_results(
    first: JoinHandle<ResultValue>,
    joiners: Vec<JoinHandle<ResultValue>>,
) -> Vec<ResultValue> {
    let mut results = vec![first.join().expect("first retention claimant completes")];
    results.extend(
        joiners
            .into_iter()
            .map(|handle| handle.join().expect("joined retention claimant completes")),
    );
    results
}

#[test]
fn parked_real_batch_has_one_claimant_and_actual_joiners_with_shared_success() {
    let mut fixture = real_fixture(2, 2);
    let root = Arc::new(root_commit(&mut fixture));
    let owner = Arc::new(fixture.owner.clone());
    let control = fixture
        ._signal_runtime
        .owner_operation_control_for_test()
        .expect("real Signal owner exposes operation control");
    let pause =
        control.arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
    let before_relational = fixture.relational_runtime.branch_basis_cost_counters();
    let before_signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");

    let first = start_claimant(Arc::clone(&owner), Arc::clone(&root));
    assert!(pause.wait_until_reached(Duration::from_secs(5)));
    let joiners = start_joiners(Arc::clone(&owner), Arc::clone(&root), 4);
    wait_for_joins(&owner, 4);
    pause.release();
    let results = collect_results(first, joiners);
    assert!(results.iter().all(Result::is_ok));
    assert_eq!(owner.unique_pin_count(), 2);
    assert_eq!(owner.active_component_obligation_count(), 10);
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

    drop(results);
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.cost_snapshot().owner_drop_releases(), 2);
    assert_eq!(
        fixture
            .relational_runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries,
        before_relational.retained_basis_registry_entries
    );
    assert_eq!(owner.unique_pin_count(), 0);
}

#[test]
fn parked_real_batch_shares_denial_rolls_back_and_retries_cleanly() {
    let mut fixture = real_fixture(2, 2);
    let root = Arc::new(root_commit(&mut fixture));
    let owner = Arc::new(fixture.owner.clone());
    let control = fixture
        ._signal_runtime
        .owner_operation_control_for_test()
        .expect("real Signal owner exposes operation control");
    let pause =
        control.arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
    control.inject_panic_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
    let before_relational = fixture.relational_runtime.branch_basis_cost_counters();
    let before_signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");

    let first = start_claimant(Arc::clone(&owner), Arc::clone(&root));
    assert!(pause.wait_until_reached(Duration::from_secs(5)));
    let joiners = start_joiners(Arc::clone(&owner), Arc::clone(&root), 4);
    wait_for_joins(&owner, 4);
    pause.release();
    let results = collect_results(first, joiners);
    assert!(results.iter().all(|result| {
        matches!(
            result,
            Err(RetentionObligationDenial::OwnerOperationPanicked)
        )
    }));
    assert_eq!(owner.unique_pin_count(), 0);
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.in_flight_acquisition_count(), 0);
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
        before_signal.retention_registry_contacts(),
        "the injected lookup panic precedes Signal's completed retention contact"
    );
    let retry = owner
        .issue_observation(&root)
        .expect("the one-shot owner panic leaves an immediate retry healthy");
    drop(retry);
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.cost_snapshot().owner_drop_releases(), 3);
    assert_eq!(owner.unique_pin_count(), 0);

    let relational = fixture.relational_runtime.branch_basis_cost_counters();
    let signal = fixture
        .signal_port
        .owner_service_cost_snapshot()
        .expect("real Signal owner remains available");
    assert_eq!(
        relational.external_retention_acquires,
        before_relational.external_retention_acquires + 2
    );
    assert_eq!(
        relational.external_retention_releases,
        before_relational.external_retention_releases + 2
    );
    assert_eq!(
        relational.retained_basis_registry_entries,
        before_relational.retained_basis_registry_entries
    );
    assert_eq!(
        signal.retention_registry_contacts(),
        before_signal.retention_registry_contacts() + 1
    );
}

#[test]
fn reserved_pair_holds_credit_until_denial_and_retries_with_the_original_reservation() {
    let fixture = real_fixture(2, 2);
    let owner = &fixture.owner;
    let control = fixture
        ._signal_runtime
        .owner_operation_control_for_test()
        .unwrap();
    let pause =
        control.arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
    control.inject_panic_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
    let reservation = owner.reserve_product_publication_pair().unwrap();
    let basis = fixture.basis.clone();
    let binding = thread::spawn(move || reservation.bind_publication(&basis));

    assert!(pause.wait_until_reached(Duration::from_secs(5)));
    // Relational has issued its lease, but the other leg can still fail.
    // Neither the lease nor its reservation credit can escape to another bind.
    assert_eq!(owner.unique_pin_count(), 2);
    assert_eq!(owner.in_flight_acquisition_count(), 2);
    assert!(owner.lock().entries.is_empty());
    assert!(matches!(
        owner.reserve_product_publication_pair(),
        Err(RetentionObligationDenial::UniquePinCapacityExhausted { .. })
    ));
    pause.release();
    let (reservation, denial) = binding.join().unwrap().unwrap_err();
    assert_eq!(denial, RetentionObligationDenial::OwnerOperationPanicked);
    assert_eq!(owner.reserved_unique_pin_capacity(), 2);
    assert_eq!(owner.reserved_in_flight_acquisition_capacity(), 2);
    assert_eq!(owner.in_flight_acquisition_count(), 0);
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.unique_pin_count(), 0);

    let retry = reservation.bind_publication(&fixture.basis).unwrap();
    assert_eq!(owner.reserved_unique_pin_capacity(), 0);
    assert_eq!(owner.reserved_in_flight_acquisition_capacity(), 0);
    drop(retry);
    assert_eq!(owner.unique_pin_count(), 0);
}

#[test]
fn evicted_signal_key_is_reserved_again_while_reacquisition_is_active() {
    let mut fixture = real_fixture(2, 2);
    let root = Arc::new(root_commit(&mut fixture));
    let owner = Arc::new(fixture.owner.clone());
    let dependency = ComponentClaimKind::AdmittedObservation;
    let seed = owner
        .issue_component(ExactComponentPinRequest::relational(
            &fixture.basis,
            dependency,
        ))
        .expect("real Relational seed claim succeeds");
    let signal = owner
        .issue_component(ExactComponentPinRequest::signal(&fixture.basis, dependency))
        .expect("real Signal seed claim succeeds");
    let original_generation = signal.lease_identity();
    drop(signal);
    assert_eq!(owner.unique_pin_count(), 1);

    let control = fixture
        ._signal_runtime
        .owner_operation_control_for_test()
        .expect("real Signal owner exposes operation control");
    let pause =
        control.arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
    let batch = start_claimant(Arc::clone(&owner), Arc::clone(&root));
    assert!(pause.wait_until_reached(Duration::from_secs(5)));

    assert_eq!(owner.unique_pin_count(), 2);
    assert_eq!(owner.in_flight_acquisition_count(), 1);
    assert_eq!(owner.active_component_obligation_count(), 3);
    let key = ExactComponentPinRequest::signal(&fixture.basis, dependency).key();
    {
        let state = owner.lock();
        assert!(!state.entries.contains_key(&key));
        assert!(state.flights.contains_key(&key));
    }

    pause.release();
    let batch = batch
        .join()
        .expect("batch retention claimant completes")
        .expect("batch retention claimant settles successfully");
    assert_ne!(batch.signal().lease_identity(), original_generation);
    drop(batch);
    drop(seed);
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.in_flight_acquisition_count(), 0);
    assert_eq!(owner.unique_pin_count(), 0);
}

#[test]
fn pair_waits_for_release_then_checks_both_acquisition_slots_before_contact() {
    let mut fixture = real_fixture(2, 1);
    let root = Arc::new(root_commit(&mut fixture));
    let owner = Arc::new(fixture.owner.clone());
    let signal = owner
        .issue_component(ExactComponentPinRequest::signal(
            &fixture.basis,
            ComponentClaimKind::AdmittedObservation,
        ))
        .unwrap();
    let before = fixture.relational_runtime.branch_basis_cost_counters();
    let control = fixture
        ._signal_runtime
        .owner_operation_control_for_test()
        .unwrap();
    let pause = control
        .arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission);
    let release = thread::spawn(move || signal.try_release());
    assert!(pause.wait_until_reached(Duration::from_secs(5)));
    assert_eq!(owner.unique_pin_count(), 1);
    let claimant = start_claimant(Arc::clone(&owner), root);
    wait_for_joins(&owner, 1);
    assert_eq!(owner.in_flight_acquisition_count(), 0);
    pause.release();
    assert_eq!(
        release.join().unwrap().unwrap().outcome(),
        crate::retention::component_obligation::ComponentBasisReleaseOutcome::OwnerReleased
    );
    assert!(matches!(
        claimant.join().unwrap(),
        Err(
            RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
                maximum_in_flight_reservations: Some(1),
            }
        )
    ));
    assert_eq!(
        fixture.relational_runtime.branch_basis_cost_counters(),
        before
    );
    assert_eq!(owner.unique_pin_count(), 0);
    assert_eq!(owner.active_observation_count(), 0);
}
