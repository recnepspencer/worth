use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use worth_signal::facade::branch::SignalOwnerOperationBoundary;

use crate::history::CompositeRuntimeWorldCommit;
use crate::retention::component_obligation::ObservationRetentionObligation;
use crate::retention::registry::{RetentionObligationDenial, RuntimeWorldRetentionOwner};
use crate::retention::unique_component_pin::ExactComponentPinRequest;
use crate::retention::ComponentBasisDependencyClass;

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
        .owner_operation_control()
        .expect("real Signal owner exposes operation control");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
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
    let costs = owner.cost_snapshot();
    assert_eq!(costs.batch_admitted(), 5);
    assert_eq!(costs.batch_denied(), 0);
    assert_eq!(costs.flights_started(), 2);
    assert_eq!(costs.single_flight_joins(), 4);
    assert_eq!(costs.relational_contacts(), 1);
    assert_eq!(costs.signal_contacts(), 1);
    assert_eq!(costs.relational_successes(), 1);
    assert_eq!(costs.signal_successes(), 1);
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
    assert_eq!(owner.reclaim(2).reclaimed(), 2);
}

#[test]
fn parked_real_batch_shares_denial_rolls_back_and_retries_cleanly() {
    let mut fixture = real_fixture(2, 2);
    let root = Arc::new(root_commit(&mut fixture));
    let owner = Arc::new(fixture.owner.clone());
    let control = fixture
        ._signal_runtime
        .owner_operation_control()
        .expect("real Signal owner exposes operation control");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    control.inject_panic_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
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
    assert_eq!(
        owner.unique_pin_count(),
        1,
        "the released Relational key is a reclaimable tombstone, not a live residue"
    );
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.in_flight_acquisition_count(), 0);
    let denied = owner.cost_snapshot();
    assert_eq!(denied.batch_admitted(), 0);
    assert_eq!(denied.batch_denied(), 5);
    assert_eq!(denied.flights_started(), 2);
    assert_eq!(denied.single_flight_joins(), 4);
    assert_eq!(denied.relational_contacts(), 1);
    assert_eq!(denied.signal_contacts(), 1);
    assert_eq!(denied.relational_successes(), 1);
    assert_eq!(denied.signal_denials(), 1);
    assert_eq!(denied.rollbacks(), 6);
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
    assert_eq!(owner.cost_snapshot().batch_admitted(), 1);
    assert_eq!(owner.cost_snapshot().relational_contacts(), 2);
    assert_eq!(owner.cost_snapshot().signal_contacts(), 2);
    drop(retry);
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.cost_snapshot().owner_drop_releases(), 3);
    assert_eq!(owner.reclaim(2).reclaimed(), 2);

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
fn reserved_pair_owner_denial_preserves_credit_without_unwind_panic() {
    let fixture = real_fixture(4, 4);
    let control = fixture
        ._signal_runtime
        .owner_operation_control()
        .expect("real Signal owner exposes operation control");
    control.inject_panic_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    let reservation = fixture
        .owner
        .reserve_product_publication_pair()
        .expect("the owner reserves the pair before contact");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        reservation.bind_publication(&fixture.basis)
    }))
    .expect("typed owner denial must not panic while releasing capacity");

    let (reservation, denial) = result.expect_err("the injected owner panic is typed");
    assert_eq!(denial, RetentionObligationDenial::OwnerOperationPanicked);
    assert_eq!(fixture.owner.reserved_unique_pin_capacity(), 2);
    assert_eq!(fixture.owner.reserved_in_flight_acquisition_capacity(), 2);
    assert_eq!(fixture.owner.in_flight_acquisition_count(), 0);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.reclaim(2).reclaimed(), 1);
    drop(reservation);
    assert_eq!(fixture.owner.reserved_unique_pin_capacity(), 0);
    assert_eq!(fixture.owner.reserved_in_flight_acquisition_capacity(), 0);

    let retry = fixture
        .owner
        .reserve_product_publication_pair()
        .expect("the consumed denial leaves reservation capacity reusable")
        .bind_publication(&fixture.basis)
        .expect("the one-shot owner denial leaves binding retryable");
    drop(retry);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
}

#[test]
fn reclaim_skips_husk_while_batch_reacquisition_flight_is_active() {
    let mut fixture = real_fixture(2, 2);
    let root = Arc::new(root_commit(&mut fixture));
    let owner = Arc::new(fixture.owner.clone());
    let dependency = ComponentBasisDependencyClass::AdmittedObservation;
    let seed = owner
        .issue_component(ExactComponentPinRequest::relational(
            &fixture.basis,
            dependency,
        ))
        .expect("real Relational seed claim succeeds");
    let signal = owner
        .issue_component(ExactComponentPinRequest::signal(&fixture.basis, dependency))
        .expect("real Signal seed claim succeeds");
    drop(signal);

    let control = fixture
        ._signal_runtime
        .owner_operation_control()
        .expect("real Signal owner exposes operation control");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
    let batch = start_claimant(Arc::clone(&owner), Arc::clone(&root));
    assert!(pause.wait_until_reached(Duration::from_secs(5)));

    assert_eq!(owner.unique_pin_count(), 2);
    assert_eq!(owner.in_flight_acquisition_count(), 1);
    assert_eq!(owner.active_component_obligation_count(), 3);
    let report = owner.reclaim(2);
    assert_eq!(report.examined(), 2);
    assert_eq!(report.reclaimed(), 0);
    assert_eq!(report.remaining_unique_pins(), 2);

    pause.release();
    let batch = batch
        .join()
        .expect("batch retention claimant completes")
        .expect("batch retention claimant settles successfully");
    drop(batch);
    drop(seed);
    assert_eq!(owner.active_component_obligation_count(), 0);
    assert_eq!(owner.in_flight_acquisition_count(), 0);
    assert_eq!(owner.reclaim(2).reclaimed(), 2);
}
