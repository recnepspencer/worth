use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

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
    RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldObservationService,
    RuntimeWorldOwnerExecutionService, RuntimeWorldPreparationService,
    RuntimeWorldProductPublicationService,
};
use crate::publication::{
    CompositeLateCancellationPosture, CompositePublicationIntent, NoEffectCompositePublication,
    OwnerExecutionOutcome, OwnerExecutionSettlement, PreparedCompositePublicationWithSignal,
    PreparedCompositePublicationWithoutSignal, RelationalAttemptProgressPosture,
    RuntimeWorldCancellationSource, SignalAttemptProgressPosture,
};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedNextAction};

use super::rehearsal::{ExecutionRehearsal, ExecutionRehearsalBoundary, ReachedExecutionBoundary};

#[path = "tests/active_custody.rs"]
mod active_custody;
#[path = "tests/between_owners.rs"]
mod between_owners;
#[path = "tests/boundaries.rs"]
mod boundaries;
#[path = "tests/cancellation.rs"]
mod cancellation;
#[path = "tests/custody_final_cas.rs"]
mod custody_final_cas;
#[path = "tests/custody_phase_drop.rs"]
mod custody_phase_drop;
#[path = "tests/failures.rs"]
mod failures;
#[path = "tests/mixed_plans.rs"]
mod mixed_plans;
#[path = "tests/planning.rs"]
mod planning;
#[path = "tests/retained_next_actions.rs"]
mod retained_next_actions;

/// A rehearsal handshake is never an open-ended wait: a boundary the owner does
/// not reach fails the arming test by name inside this budget.
const REHEARSAL_HANDSHAKE_BUDGET: Duration = Duration::from_secs(5);

type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

#[derive(Clone, Copy)]
struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(0)
    }
}

#[derive(Clone)]
struct MutableClock {
    ticks: Arc<AtomicU64>,
}

impl MutableClock {
    fn new(ticks: u64) -> Self {
        Self {
            ticks: Arc::new(AtomicU64::new(ticks)),
        }
    }

    fn set(&self, ticks: u64) {
        self.ticks.store(ticks, Ordering::Release);
    }
}

impl RuntimeWorldClockSource for MutableClock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(self.ticks.load(Ordering::Acquire))
    }
}

fn budgets(publication_attempts: u64) -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 12,
            // Covers the declared history count including canonical publication envelopes.
            history_metadata_bytes: 65536,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 4,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: publication_attempts,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 4,
            retained_partial_metadata_bytes:
                crate::recovery::ProductUnpublishedOwnerEffects::metadata_charge_hint()
                    .saturating_mul(publication_attempts as usize) as u64,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 12,
            in_flight_pin_acquisition_reservations: 12,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 4,
        },
    })
    .expect("focused execution budgets are positive")
}

fn setup() -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    setup_with_clock(RuntimeWorldClock::from_source(FixedClock))
}

fn setup_with_relational_source() -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    let (fixture, owner, expected) = setup();
    let warmup = settled(execute_without_signal(
        &owner,
        prepare_relational(&fixture, &owner, &expected, "relational-source-seed"),
    ));
    let successor = warmup
        .successor_basis()
        .cloned()
        .expect("the canonical seed carries its successor basis");
    let ready = warmup
        .ready(successor)
        .expect("the canonical seed forms a product publication");
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    match RuntimeWorldProductPublicationService::publish(
        owner.as_ref(),
        ready,
        &cell,
        CompositeLateCancellationPosture::NotRequested,
    ) {
        crate::publication::RuntimeWorldPublicationOutcome::Performed(_) => {}
        other => panic!("the canonical seed publishes its product head: {other:?}"),
    }
    let expected = RuntimeWorldObservationService::observe_product_branch(
        owner.as_ref(),
        &expected.branch_identity().clone(),
    )
    .expect("the owner re-observes the published source basis");
    (fixture, owner, expected)
}

fn setup_with_clock(
    clock: RuntimeWorldClock,
) -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    let mut fixture = reference_test_fixture::real_fixture(12, 12);
    let owner = Arc::new(
        TestOwner::new(fixture.owner_inputs(budgets(4), clock))
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

/// One Relational publication off the observed head. Preparation and
/// reservation are one owner step, so no test can hold a lowered plan without
/// the capacity that authorizes executing it.
fn prepare_relational(
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    operation_name: &str,
) -> PreparedCompositePublicationWithoutSignal {
    let cancellation = RuntimeWorldCancellationSource::new();
    RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected.clone(),
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(operation_name),
            ),
        &cancellation.token(),
        None,
    )
    .expect("the exact observed head admits the requested Relational plan")
}

/// One Signal-advancing publication, optionally alongside a Relational change.
fn prepare_signal(
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    relational: Option<RelationalTransactionIntent>,
) -> PreparedCompositePublicationWithSignal {
    prepare_signal_with_deadline(owner, expected, relational, None)
        .expect("the exact observed head admits the requested Signal plan")
}

fn prepare_signal_with_deadline(
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    relational: Option<RelationalTransactionIntent>,
    deadline: Option<crate::lifecycle::RuntimeWorldInstant>,
) -> Result<PreparedCompositePublicationWithSignal, NoEffectCompositePublication> {
    let cancellation = RuntimeWorldCancellationSource::new();
    RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected.clone(),
        CompositePublicationIntent::with_signal(relational),
        &cancellation.token(),
        deadline,
    )
}

/// A second, independently prepared Relational publication off the same head.
/// Two real owner effects race for one product head; only one CAS wins.
fn ready_relational_competitor(
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    operation_name: &str,
) -> crate::publication::CompositePublicationReady {
    let settlement = settled(execute_without_signal(
        owner,
        prepare_relational(fixture, owner, expected, operation_name),
    ));
    let successor = settlement
        .successor_basis()
        .cloned()
        .expect("the canonical competitor returns its successor basis");
    settlement
        .ready(successor)
        .expect("the canonical competitor forms a ready publication")
}

fn publish_ready_competing_head(
    owner: &TestOwner,
    ready: crate::publication::CompositePublicationReady,
    expected: &ProductBranchObservation,
) -> ProductBranchObservation {
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    match RuntimeWorldProductPublicationService::publish(
        owner,
        ready,
        &cell,
        CompositeLateCancellationPosture::NotRequested,
    ) {
        crate::publication::RuntimeWorldPublicationOutcome::Performed(_) => {}
        other => panic!("the canonical competing publication must perform: {other:?}"),
    }
    RuntimeWorldObservationService::observe_product_branch(
        owner,
        &expected.branch_identity().clone(),
    )
    .expect("the owner observes the canonical competing product head")
}

fn execute_without_signal(
    owner: &TestOwner,
    prepared: PreparedCompositePublicationWithoutSignal,
) -> OwnerExecutionOutcome {
    let cancellation = RuntimeWorldCancellationSource::new();
    RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner,
        prepared,
        &cancellation.token(),
    )
}

fn execute_with_empty_signal(
    owner: &TestOwner,
    prepared: PreparedCompositePublicationWithSignal,
) -> OwnerExecutionOutcome {
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut context = ();
    RuntimeWorldOwnerExecutionService::execute_with_signal(
        owner,
        prepared,
        &mut context,
        &cancellation.token(),
        |_| Ok(()),
    )
}

fn settled(outcome: OwnerExecutionOutcome) -> OwnerExecutionSettlement {
    match outcome {
        OwnerExecutionOutcome::Settled(settlement) => settlement,
        other => panic!("owner execution must settle: {other:?}"),
    }
}

/// Arms one execution boundary for this owner and returns the arming handle
/// together with the channel the parked execution reports on.
fn arm_rehearsal(
    owner: &TestOwner,
    boundary: ExecutionRehearsalBoundary,
) -> (ExecutionRehearsal, Receiver<ReachedExecutionBoundary>) {
    let (sender, receiver) = sync_channel(1);
    (
        ExecutionRehearsal::arm(owner.owner_identity(), boundary, sender),
        receiver,
    )
}

fn await_boundary(receiver: &Receiver<ReachedExecutionBoundary>) -> ReachedExecutionBoundary {
    receiver
        .recv_timeout(REHEARSAL_HANDSHAKE_BUDGET)
        .expect("the owner reaches the armed execution boundary within its budget")
}

/// The exact Signal token the seam is about to hand the Signal owner.
fn awaited_signal_token(
    receiver: &Receiver<ReachedExecutionBoundary>,
) -> worth_signal::facade::branch::SignalOwnerCancellationToken {
    match await_boundary(receiver) {
        ReachedExecutionBoundary::SignalAdvance { signal_token } => signal_token,
        ReachedExecutionBoundary::BetweenOwnerEffects => {
            panic!("the armed boundary is the Signal advance seam")
        }
    }
}

/// One publication that changes both owners: Relational `PublishPrepared` and
/// then Signal `AdvanceExact`, in the one canonical order.
fn prepare_both_owners(
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    operation_name: &str,
) -> PreparedCompositePublicationWithSignal {
    let cancellation = RuntimeWorldCancellationSource::new();
    RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected.clone(),
        CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(operation_name),
            ),
        &cancellation.token(),
        None,
    )
    .expect("the exact observed head admits both owner legs")
}

fn retained(outcome: OwnerExecutionOutcome) -> crate::recovery::ProductUnpublishedOwnerEffects {
    match outcome {
        OwnerExecutionOutcome::ProductUnpublished(retained) => retained,
        other => panic!("the attempt must retain its owner effects: {other:?}"),
    }
}

/// The exact record shape every proof in this module expects when a settled
/// Relational effect is retained and the Signal owner never moved.
fn assert_retains_only_the_relational_effect(
    record: &crate::recovery::ProductUnpublishedOwnerEffects,
    cause: ProductUnpublishedCause,
) {
    assert_eq!(record.cause(), cause);
    assert_eq!(record.owner_effect_count(), 1);
    assert_eq!(
        record.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(
        record.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
}

#[path = "tests/public_cutoff.rs"]
mod public_cutoff;
#[path = "tests/public_ports.rs"]
mod public_ports;

#[path = "tests/public_inspection.rs"]
mod public_inspection;
