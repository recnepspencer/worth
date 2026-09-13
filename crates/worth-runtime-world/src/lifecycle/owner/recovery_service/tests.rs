//! Settlement and recovery-catalog custody tests only. These tests manually
//! supply owner-issued effects to prove custody, not production dispatch.

use std::sync::Arc;

use worth_relational::facade::mvcc::RelationalTransactionIntent;

use crate::branch::reference_test_fixture::{self, RealReferenceFixture};
use crate::branch::{ProductBranchObservation, RuntimeWorldBootstrapOutcome};
use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::lifecycle::{
    RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldPreparationService,
    RuntimeWorldRecoveryService,
};
use crate::publication::{
    CompositeAttemptProgress, CompositePublicationIntent, RelationalAttemptProgress,
    ReservedCompositePublicationAttempt, RuntimeWorldCancellationSource, SignalAttemptProgress,
};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedNextAction};

pub(super) type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(7)
    }
}

fn budgets(maximum_recovery_records: u64) -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 4,
            history_metadata_bytes: 4096,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 1,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 2,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: maximum_recovery_records,
            retained_partial_metadata_bytes:
                crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint() as u64,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 6,
            in_flight_pin_acquisition_reservations: 4,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 1,
        },
    })
    .expect("recovery tests install positive owner budgets")
}

fn setup_with_recovery_limit(
    maximum_recovery_records: u64,
) -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    let mut fixture = reference_test_fixture::real_fixture(8, 8);
    let owner = Arc::new(
        TestOwner::new(fixture.owner_inputs(
            budgets(maximum_recovery_records),
            RuntimeWorldClock::from_source(FixedClock),
        ))
        .expect("managed owner construction"),
    );
    let performed = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    (fixture, owner, performed.product_branch().clone())
}

pub(super) fn setup() -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    setup_with_recovery_limit(1)
}

/// One Relational reservation off the observed head. Lowering and reservation
/// are a single owner step; these custody tests then supply owner-issued
/// effects by hand rather than driving production dispatch.
pub(super) fn relational_attempt(
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: ProductBranchObservation,
) -> ReservedCompositePublicationAttempt {
    let cancellation = RuntimeWorldCancellationSource::new();
    RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected,
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate("recovery-custody"),
            ),
        &cancellation.token(),
        None,
    )
    .expect("the owner prepares and reserves the exact relational recovery attempt")
    .into_attempt()
}

pub(super) fn successor_basis(
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    relational: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    signal: Option<worth_signal::facade::branch::AdmittedSignalBranchBasis>,
) -> crate::basis::AdmittedCompositeRuntimeWorldBasis {
    crate::basis::admit_current(
        &owner
            .state
            .identities
            .lock()
            .unwrap_or_else(|error| error.into_inner()),
        &owner.state.relational.basis_port(),
        &owner.state.signal.basis_port(),
        &owner.state.bridge,
        relational,
        signal.unwrap_or_else(|| expected.basis().signal_basis().clone()),
        expected.basis().correspondence_basis().clone(),
    )
    .expect("the real component owners admit the exact successor tuple")
}

#[test]
fn caller_loss_preserves_relational_settlement_custody_and_catalog_capacity() {
    let (fixture, owner, expected) = setup();
    let mut attempt = relational_attempt(&fixture, &owner, expected.clone());
    attempt.begin_owner_execution();

    let performed = fixture.perform_relational_owner_change();
    let successor_basis = successor_basis(&owner, &expected, performed.next_basis().clone(), None);
    let progress = CompositeAttemptProgress::new(
        RelationalAttemptProgress::performed(performed),
        SignalAttemptProgress::untouched(),
    );
    let retained = attempt
        .settle(progress)
        .ready(successor_basis)
        .expect_err("unsettled Relational work enters retained recovery");

    assert_eq!(retained.cause(), ProductUnpublishedCause::SettlementPending);
    let handle = retained.recovery_handle();
    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(owner.recovery_handles(), vec![handle.clone()]);
    // SPEC-P4-008: an installed record is exposed by close, not refused, and
    // this proof still needs its world open. See
    // `close_exposes_every_retained_record_in_its_terminal_report`.
    drop(retained);

    assert_eq!(owner.recovery_record_count(), 1);
    let inspected = owner
        .inspect_recovery(&handle)
        .expect("caller loss leaves the catalog record inspectable");
    assert_eq!(
        inspected.cause(),
        ProductUnpublishedCause::SettlementPending
    );
    assert!(owner.cleanup_recovery_handle(&handle).is_none());
    drop(inspected);
    assert!(owner.cleanup_recovery_handle(&handle).is_none());
    assert!(matches!(
        owner
            .state
            .recovery
            .reserve_product_unpublished(owner.owner_identity()),
        Err(crate::recovery::RecoveryCatalogDenial::CapacityExhausted { maximum: 1 })
    ));
}

#[test]
fn continuation_settles_relational_effects_without_signal_or_product_publication() {
    let (fixture, owner, expected) = setup();
    let mut attempt = relational_attempt(&fixture, &owner, expected.clone());
    attempt.begin_owner_execution();

    let before_product_head = owner
        .state
        .branches
        .root_cell()
        .expect("bootstrapped cell")
        .atomic_snapshot();
    let performed = fixture.perform_relational_owner_change();
    let successor_basis = successor_basis(&owner, &expected, performed.next_basis().clone(), None);
    let handle = {
        let progress = CompositeAttemptProgress::new(
            RelationalAttemptProgress::performed(performed),
            SignalAttemptProgress::untouched(),
        );
        attempt
            .settle(progress)
            .ready(successor_basis)
            .expect_err("performed Relational work remains a recovery record until settled")
            .recovery_handle()
    };
    let continuation = RuntimeWorldRecoveryService::continue_effects(
        owner.as_ref(),
        owner
            .inspect_recovery(&handle)
            .expect("catalog exposes the retained caller capability"),
    )
    .expect("the real Relational settlement authority completes recovery");
    assert!(!continuation
        .actions()
        .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
    assert!(continuation
        .actions()
        .contains(&ProductUnpublishedNextAction::ReleaseObligations));
    let inspected = owner
        .inspect_recovery(&handle)
        .expect("settled recovery remains inspectable until explicit cleanup");
    assert_eq!(
        inspected.cause(),
        ProductUnpublishedCause::SettlementPending,
        "settlement completion updates progress/actions without rewriting provenance"
    );
    assert_eq!(
        inspected.progress().relational_posture(),
        crate::publication::RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(
        inspected.progress().signal_posture(),
        crate::publication::SignalAttemptProgressPosture::Untouched
    );
    drop(inspected);
    assert_eq!(
        owner
            .state
            .branches
            .root_cell()
            .expect("bootstrapped cell")
            .atomic_snapshot(),
        before_product_head,
        "recovery settlement does not publish a product head"
    );
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.recovery_record_count(), 0);
}

#[cfg(feature = "test-operation-control")]
#[test]
fn metadata_ceiling_rejects_second_charge_and_cleanup_releases_the_first() {
    let (mut fixture, owner, expected) = setup_with_recovery_limit(2);
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut attempt = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        CompositePublicationIntent::with_signal(None),
        &cancellation.token(),
        None,
    )
    .expect("the owner prepares and reserves the exact signal recovery attempt")
    .into_attempt();
    attempt.begin_owner_execution();

    let advanced = fixture.perform_signal_owner_change();
    let successor_basis = successor_basis(
        &owner,
        &expected,
        expected.basis().relational_basis().clone(),
        Some(advanced.advanced_basis().clone()),
    );
    fixture.inject_signal_retention_panic();
    let progress = CompositeAttemptProgress::new(
        RelationalAttemptProgress::untouched(),
        SignalAttemptProgress::advanced(advanced),
    );
    let retained = attempt
        .settle(progress)
        .ready(successor_basis)
        .expect_err("post-effect retention denial enters catalogued recovery");
    assert!(!retained
        .next_actions()
        .contains(&ProductUnpublishedNextAction::SettleOwnerEffects));
    let first_charge = retained.metadata_bytes();
    assert_eq!(
        first_charge,
        crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint()
    );
    owner
        .state
        .recovery
        .set_metadata_ceiling_for_test(first_charge);
    let handle = retained.recovery_handle();
    drop(retained);
    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(owner.state.recovery.metadata_bytes(), first_charge);
    assert!(matches!(
        owner
            .state
            .recovery
            .reserve_product_unpublished(owner.owner_identity()),
        Err(crate::recovery::RecoveryCatalogDenial::CapacityExhausted { .. })
    ));
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.recovery_record_count(), 0);
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
    let released_slot = owner
        .state
        .recovery
        .reserve_product_unpublished(owner.owner_identity())
        .expect("eligible cleanup releases the exact metadata charge");
    drop(released_slot);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(
        owner
            .state
            .retention
            .reserved_in_flight_acquisition_capacity(),
        0
    );
}
