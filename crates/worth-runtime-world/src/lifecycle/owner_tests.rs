use crate::branch::{RuntimeWorldBootstrapNoEffectCause, RuntimeWorldBootstrapOutcome};
use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::identity::RuntimeWorldIdentityFamily;
use crate::lifecycle::{
    RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldCloseDenial,
    RuntimeWorldOwnerLifecycleObservation,
};

use super::RuntimeWorldOwnerConstructionContract;

#[path = "owner_tests/admission_race.rs"]
mod admission_race;
#[path = "owner_tests/close_report.rs"]
mod close_report;
#[path = "owner_tests/product_cas_loss.rs"]
mod product_cas_loss;
#[path = "owner_tests/publication.rs"]
mod publication;
#[path = "owner_tests/recovery_metadata_budget.rs"]
mod recovery_metadata_budget;

#[test]
fn owner_construction_owns_one_non_resettable_issuer() {
    let first = RuntimeWorldOwnerConstructionContract::new().expect("first managed owner identity");
    let second =
        RuntimeWorldOwnerConstructionContract::new().expect("second managed owner identity");
    assert_ne!(first.owner_identity(), second.owner_identity());
}

#[test]
fn owner_issuer_keeps_families_scoped_and_checked() {
    let mut construction = RuntimeWorldOwnerConstructionContract::new().expect("owner identity");
    let owner = construction.owner_identity();
    let issuer = construction.issuer_mut();
    assert_eq!(issuer.owner(), owner);
    assert_eq!(issuer.branch_incarnation().unwrap().owner_identity(), owner);
    assert_eq!(issuer.composite_commit().unwrap().owner_identity(), owner);
    assert_eq!(issuer.bootstrap_attempt().unwrap().owner_identity(), owner);
    assert_eq!(
        issuer.publication_attempt().unwrap().owner_identity(),
        owner
    );
    assert_eq!(
        issuer.product_unpublished().unwrap().owner_identity(),
        owner
    );
    assert_ne!(
        issuer.composite_commit().unwrap(),
        issuer.composite_commit().unwrap()
    );

    issuer.set_next_publication_attempt_for_test(u64::MAX);
    let denial = issuer
        .publication_attempt()
        .expect_err("the checked sequence must not wrap");
    assert_eq!(
        denial.family(),
        RuntimeWorldIdentityFamily::PublicationAttempt
    );
    assert!(issuer.publication_attempt().is_err());
}

struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(7)
    }
}

fn bootstrap_budgets() -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 4,
            // This fixture tests lifecycle limits, with room for performed history facts.
            history_metadata_bytes: 16384,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 2,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 2,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 2,
            retained_partial_metadata_bytes:
                crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint()
                    .saturating_mul(2) as u64,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            // Two bootstrap pins plus two pessimistic pin reservations for
            // each of the two concurrently admitted publication attempts.
            unique_exact_component_pins: 6,
            in_flight_pin_acquisition_reservations: 4,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 1,
        },
    })
    .expect("bootstrap budgets are positive")
}

#[test]
fn bootstrap_installs_one_coherent_root_and_second_call_is_no_contact() {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let owner = super::RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        bootstrap_budgets(),
        RuntimeWorldClock::from_source(FixedClock),
    ))
    .expect("managed owner construction");
    let intent = fixture.bootstrap_intent();
    let performed = match owner.bootstrap_root(intent.clone()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };

    assert_eq!(performed.basis().owner_identity(), owner.owner_identity());
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );
    assert_eq!(owner.state.history.len(), 1);
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert!(owner.state.branches.root_cell().is_some());
    assert_eq!(owner.state.retention.unique_pin_count(), 2);
    assert_eq!(owner.state.retention.active_component_obligation_count(), 6);

    let before_retention = owner.state.retention.cost_snapshot();
    let before_history = owner.state.history.counters();
    let second = owner.bootstrap_root(intent);
    assert!(matches!(
        second,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect)
            if no_effect.cause() == RuntimeWorldBootstrapNoEffectCause::AlreadyBootstrapped
    ));
    assert_eq!(owner.state.retention.cost_snapshot(), before_retention);
    assert_eq!(owner.state.history.counters(), before_history);
}

#[test]
fn failed_basis_admission_restores_root_capacity_and_allows_retry() {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let foreign = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let owner = super::RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        bootstrap_budgets(),
        RuntimeWorldClock::from_source(FixedClock),
    ))
    .expect("managed owner construction");
    let before_retention = owner.state.retention.cost_snapshot();
    let before_history = owner.state.history.counters();
    let denied = owner.bootstrap_root(foreign.bootstrap_intent());
    assert!(matches!(
        denied,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect)
            if no_effect.cause() == RuntimeWorldBootstrapNoEffectCause::ForeignBasis
    ));
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Open
    );
    assert_eq!(owner.state.history.len(), 0);
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.retention.active_component_obligation_count(), 0);
    assert_eq!(owner.state.retention.cost_snapshot(), before_retention);
    assert_eq!(owner.state.history.counters(), before_history);
    assert!(owner.state.branches.root_cell().is_none());

    assert!(matches!(
        owner.bootstrap_root(fixture.bootstrap_intent()),
        RuntimeWorldBootstrapOutcome::Performed(_)
    ));
}

#[test]
fn close_is_typed_and_prevents_a_later_bootstrap() {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let owner = super::RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        bootstrap_budgets(),
        RuntimeWorldClock::from_source(FixedClock),
    ))
    .expect("managed owner construction");
    assert!(owner.close().is_ok());
    assert_eq!(
        owner.lifecycle_observation(),
        RuntimeWorldOwnerLifecycleObservation::Closed
    );
    assert_eq!(
        owner
            .close()
            .expect_err("a closed owner cannot close twice"),
        RuntimeWorldCloseDenial::AlreadyClosed
    );
    assert!(matches!(
        owner.bootstrap_root(fixture.bootstrap_intent()),
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect)
            if no_effect.cause() == RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable
    ));
}

#[test]
fn closed_owner_rejects_preparation_without_consuming_capacity() {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let owner = super::RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        bootstrap_budgets(),
        RuntimeWorldClock::from_source(FixedClock),
    ))
    .expect("managed owner construction");
    let performed = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    let expected = performed.product_branch().clone();
    let before = (
        owner.state.history.reserved_len(),
        owner.state.recovery.reserved_slots(),
        owner.state.retention.reserved_unique_pin_capacity(),
        owner.state.publication_capacity.active(),
    );
    let _report = owner.close().expect("owner closes while idle");
    let denied = crate::lifecycle::RuntimeWorldPreparationService::prepare_publication(
        &owner,
        expected,
        crate::publication::CompositePublicationIntent::with_signal(None),
        &crate::publication::RuntimeWorldCancellationSource::new().token(),
        None,
    )
    .expect_err("closed owner cannot prepare a publication");
    assert_eq!(
        denied.cause(),
        crate::publication::NoEffectCause::OwnerUnavailable
    );
    assert_eq!(
        (
            owner.state.history.reserved_len(),
            owner.state.recovery.reserved_slots(),
            owner.state.retention.reserved_unique_pin_capacity(),
            owner.state.publication_capacity.active(),
        ),
        before
    );
}

#[test]
fn owner_reservation_owns_all_real_capacity_and_drops_it_as_one_attempt() {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let owner = super::RuntimeWorldOwnerRoot::new(fixture.owner_inputs(
        bootstrap_budgets(),
        RuntimeWorldClock::from_source(FixedClock),
    ))
    .expect("managed owner construction");
    let performed = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    let cancellation = crate::publication::RuntimeWorldCancellationSource::new();
    let attempt = crate::lifecycle::RuntimeWorldPreparationService::prepare_publication(
        &owner,
        performed.product_branch().clone(),
        crate::publication::CompositePublicationIntent::with_signal(None),
        &cancellation.token(),
        None,
    )
    .expect("owner issues the complete reservation bundle");

    assert_eq!(owner.state.history.reserved_len(), 1);
    assert_eq!(owner.state.recovery.reserved_slots(), 1);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 2);
    assert_eq!(
        owner
            .state
            .retention
            .reserved_in_flight_acquisition_capacity(),
        2
    );
    assert_eq!(owner.state.publication_capacity.active(), 1);
    assert_eq!(owner.state.operation.active(), 1);
    assert_eq!(
        owner
            .close()
            .expect_err("a live attempt blocks the close drain"),
        RuntimeWorldCloseDenial::AlreadyClosing
    );
    drop(attempt);
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(
        owner
            .state
            .retention
            .reserved_in_flight_acquisition_capacity(),
        0
    );
    assert_eq!(owner.state.publication_capacity.active(), 0);
    assert_eq!(owner.state.operation.active(), 0);
}

#[path = "owner_tests/bootstrap_boundary.rs"]
mod bootstrap_boundary;
