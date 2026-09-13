use std::sync::{Arc, Barrier};
use std::thread;

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;
use worth_signal::facade::{SignalGraph, SignalRuntime};

use crate::identity::RuntimeWorldOwnerIdentity;
use crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract;
use crate::retention::component_obligation::{
    ComponentBasisPinObligation, ComponentBasisReleaseOutcome, RetentionReleaseDenial,
};
use crate::retention::obligation_transfer::ComponentBasisObligationTransferDestination;
use crate::retention::registry::{RetentionObligationDenial, RuntimeWorldRetentionOwner};

use super::{ComponentBasisDependencyClass, ExactComponentBasis, ExactComponentPinRequest};

struct Fixture {
    owner: RuntimeWorldRetentionOwner<(), (), ()>,
    owner_identity: RuntimeWorldOwnerIdentity,
    relational: AdmittedRelationalBranchBasis,
    signal: AdmittedSignalBranchBasis,
    _relational_runtime: Arc<worth_relational::facade::runtime::RelationalRuntime>,
    _signal_runtime: SignalRuntime<(), (), (), (), ()>,
}

fn fixture(unique: u64, reservations: u64) -> Fixture {
    let relational_runtime = Arc::new(RelationalRuntimeApi::builder().build());
    let relational_port = relational_runtime.owner_component_services().basis_port();
    let relational_identity = relational_runtime.main_branch_identity();
    let (_, relational) = relational_runtime
        .observe_branch(&relational_identity)
        .expect("the real Relational owner admits its main basis");
    let mut signal_runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let signal_port = signal_runtime
        .owner_component_services()
        .expect("the real Signal owner issues its basis service")
        .basis_port();
    let signal = signal_runtime
        .observe_signal_branch_basis(signal_runtime.current_branch())
        .expect("the real Signal owner admits its current basis");
    let owner_identity = RuntimeWorldOwnerConstructionContract::new()
        .expect("the Runtime World owner identity is checked")
        .owner_identity();
    let budgets = crate::budget::RuntimeWorldBudgets::install(
        crate::budget::RuntimeWorldBudgetInstallation {
            branches: crate::budget::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 1,
            },
            history: crate::budget::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1,
                history_metadata_bytes: 1,
            },
            observations: crate::budget::RuntimeWorldObservationBudgetInstallation {
                active_observations: 1,
            },
            publication: crate::budget::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 1,
            },
            recovery: crate::budget::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 1,
                retained_partial_metadata_bytes: 1,
            },
            retention: crate::budget::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: unique,
                in_flight_pin_acquisition_reservations: reservations,
            },
            custody: crate::budget::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 1,
            },
        },
    )
    .expect("retention limits are positive");
    let owner = RuntimeWorldRetentionOwner::new(
        owner_identity,
        relational_port,
        signal_port,
        budgets.unique_exact_component_pins(),
        budgets.in_flight_pin_acquisition_reservations(),
        budgets.active_observations(),
    );
    Fixture {
        owner,
        owner_identity,
        relational,
        signal,
        _relational_runtime: relational_runtime,
        _signal_runtime: signal_runtime,
    }
}

fn signal_request<'a>(
    owner: RuntimeWorldOwnerIdentity,
    basis: &'a AdmittedSignalBranchBasis,
    dependency: ComponentBasisDependencyClass,
) -> ExactComponentPinRequest<'a> {
    ExactComponentPinRequest {
        owner,
        component: ExactComponentBasis::Signal(basis),
        dependency,
    }
}

fn relational_request<'a>(
    owner: RuntimeWorldOwnerIdentity,
    basis: &'a AdmittedRelationalBranchBasis,
    dependency: ComponentBasisDependencyClass,
) -> ExactComponentPinRequest<'a> {
    ExactComponentPinRequest {
        owner,
        component: ExactComponentBasis::Relational(basis),
        dependency,
    }
}

#[test]
fn repeated_exact_use_shares_one_real_signal_lease_and_reclaims_tombstone() {
    let fixture = fixture(4, 4);
    let first = fixture
        .owner
        .issue_component(signal_request(
            fixture.owner_identity,
            &fixture.signal,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect("first exact Signal claim");
    let identity = first.lease_identity();
    let second = fixture
        .owner
        .issue_component(signal_request(
            fixture.owner_identity,
            &fixture.signal,
            ComponentBasisDependencyClass::HistoricalInspection,
        ))
        .expect("second dependency count on the same exact key");
    assert_eq!(first.key(), second.key());
    assert_eq!(first.lease_identity(), identity);
    assert_eq!(fixture.owner.unique_pin_count(), 1);
    assert_eq!(fixture.owner.active_component_obligation_count(), 2);
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        1
    );
    assert_eq!(
        second
            .try_release()
            .expect("shared count releases locally")
            .outcome(),
        ComponentBasisReleaseOutcome::SharedOwnerLease
    );
    assert_eq!(fixture.owner.cost_snapshot().owner_release_contacts(), 0);
    assert_eq!(
        first
            .try_release()
            .expect("last count releases the real lease")
            .outcome(),
        ComponentBasisReleaseOutcome::OwnerReleased
    );
    assert_eq!(fixture.owner.cost_snapshot().owner_release_contacts(), 1);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.unique_pin_count(), 1);
    let zero = fixture.owner.reclaim(0);
    assert_eq!(zero.examined(), 0);
    assert_eq!(zero.reclaimed(), 0);
    assert_eq!(zero.remaining_unique_pins(), 1);
    let report = fixture.owner.reclaim(1);
    assert_eq!(report.examined(), 1);
    assert_eq!(report.reclaimed(), 1);
    assert_eq!(report.remaining_unique_pins(), 0);
    let reopened = fixture
        .owner
        .issue_component(signal_request(
            fixture.owner_identity,
            &fixture.signal,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect("the reclaimed exact key can be opened again");
    assert_ne!(reopened.lease_identity(), identity);
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        2
    );
    assert_eq!(
        reopened
            .try_release()
            .expect("the new lease generation releases")
            .outcome(),
        ComponentBasisReleaseOutcome::OwnerReleased
    );
}

#[test]
fn all_transfers_change_counts_without_contacting_a_component_owner() {
    let fixture = fixture(4, 4);
    let mut obligation = fixture
        .owner
        .issue_component(signal_request(
            fixture.owner_identity,
            &fixture.signal,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect("transfer source claim");
    let before = fixture.owner.cost_snapshot();
    let destinations = [
        ComponentBasisObligationTransferDestination::ProductBranchHead,
        ComponentBasisObligationTransferDestination::RetainedCompositeHistory,
        ComponentBasisObligationTransferDestination::AdmittedObservation,
        ComponentBasisObligationTransferDestination::ActivePublicationAttempt,
        ComponentBasisObligationTransferDestination::ProductUnpublishedOwnerEffects,
        ComponentBasisObligationTransferDestination::HistoricalInspection,
    ];
    for destination in destinations {
        obligation = obligation
            .try_transfer_to(destination)
            .expect("a live count transfers atomically");
        assert_eq!(
            obligation.dependency(),
            destination.dependency_class().unwrap()
        );
    }
    let (obligation, denial) = obligation
        .try_transfer_to(ComponentBasisObligationTransferDestination::Release)
        .expect_err("Release is not a transfer destination");
    assert_eq!(
        denial,
        super::super::obligation_transfer::RetentionTransferDenial::ReleaseDestination
    );
    assert_eq!(
        obligation
            .try_release()
            .expect("the transferred claim releases")
            .outcome(),
        ComponentBasisReleaseOutcome::OwnerReleased
    );
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        before.owner_acquisition_contacts()
    );
    assert_eq!(fixture.owner.cost_snapshot().owner_release_contacts(), 1);
}

#[test]
fn foreign_signal_basis_cannot_substitute_into_an_exact_pin() {
    let fixture = fixture(1, 1);
    let foreign_runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let foreign_basis = foreign_runtime
        .observe_signal_branch_basis(foreign_runtime.current_branch())
        .expect("the foreign real Signal owner admits its basis");
    let denial = fixture
        .owner
        .issue_component(signal_request(
            fixture.owner_identity,
            &foreign_basis,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect_err("the local Signal owner rejects a foreign exact basis");
    assert_eq!(
        denial,
        RetentionObligationDenial::Signal(
            worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial::ForeignBasis
        )
    );
    assert_eq!(fixture.owner.unique_pin_count(), 0);
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        1
    );
}

#[test]
fn concurrent_first_use_has_one_owner_contact_and_one_key_slot() {
    let fixture = fixture(8, 16);
    let owner = Arc::new(fixture.owner);
    let basis = Arc::new(fixture.signal);
    let owner_identity = fixture.owner_identity;
    let barrier = Arc::new(Barrier::new(8));
    let mut handles = Vec::new();
    for _ in 0..8 {
        let owner = Arc::clone(&owner);
        let basis = Arc::clone(&basis);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            owner
                .issue_component(signal_request(
                    owner_identity,
                    &basis,
                    ComponentBasisDependencyClass::AdmittedObservation,
                ))
                .expect("concurrent exact use")
        }));
    }
    let claims: Vec<ComponentBasisPinObligation> = handles
        .into_iter()
        .map(|handle| handle.join().expect("claimant completes"))
        .collect();
    assert_eq!(owner.unique_pin_count(), 1);
    assert_eq!(owner.active_component_obligation_count(), 8);
    assert_eq!(owner.cost_snapshot().owner_acquisition_contacts(), 1);
    assert!(owner.cost_snapshot().single_flight_joins() <= 7);
    drop(claims);
    assert_eq!(owner.cost_snapshot().owner_drop_releases(), 1);
}

#[test]
fn foreign_owner_and_owner_loss_are_pre_effect_and_terminally_safe() {
    let fixture = fixture(1, 1);
    let foreign = RuntimeWorldOwnerConstructionContract::new()
        .expect("foreign Runtime World owner identity")
        .owner_identity();
    let denial = fixture
        .owner
        .issue_component(signal_request(
            foreign,
            &fixture.signal,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect_err("foreign ownership is rejected before owner contact");
    assert!(matches!(
        denial,
        RetentionObligationDenial::ForeignOwner { .. }
    ));
    assert_eq!(fixture.owner.unique_pin_count(), 0);
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        0
    );
    let claim = fixture
        .owner
        .issue_component(signal_request(
            fixture.owner_identity,
            &fixture.signal,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect("owner-loss claim");
    let denial = fixture
        .owner
        .issue_component(relational_request(
            fixture.owner_identity,
            &fixture.relational,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect_err("a second exact key is denied at the installed unique-pin bound");
    assert_eq!(
        denial,
        RetentionObligationDenial::UniquePinCapacityExhausted {
            maximum_unique_component_pins: 1,
        }
    );
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        1
    );
    drop(fixture._signal_runtime);
    let (claim, denial) = claim
        .try_release()
        .expect_err("a dead Signal owner preserves the claim");
    assert!(matches!(
        denial,
        RetentionReleaseDenial::Signal(
            worth_signal::facade::branch::SignalBranchRetentionReleaseDenial::OwnerUnavailable(_)
        )
    ));
    drop(claim);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    assert_eq!(fixture.owner.cost_snapshot().owner_release_contacts(), 1);
    assert_eq!(fixture.owner.cost_snapshot().owner_drop_releases(), 1);
    assert_eq!(fixture.owner.reclaim(1).reclaimed(), 1);
    let relational = fixture
        .owner
        .issue_component(relational_request(
            fixture.owner_identity,
            &fixture.relational,
            ComponentBasisDependencyClass::AdmittedObservation,
        ))
        .expect("reclamation makes the bounded slot reusable");
    assert_eq!(
        relational
            .try_release()
            .expect("the real Relational owner releases its exact lease")
            .outcome(),
        ComponentBasisReleaseOutcome::OwnerReleased
    );
    assert_eq!(fixture.owner.cost_snapshot().owner_release_contacts(), 2);
}
