use std::sync::Arc;

use worth_relational::facade::mvcc::RelationalTransactionIntent;

use crate::branch::reference_test_fixture::{self, RealReferenceFixture};
use crate::branch::ProductBranchObservation;
use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::lifecycle::{
    RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldCloseDenial,
    RuntimeWorldOwnerExecutionService, RuntimeWorldPreparationService,
};

use crate::publication::{
    CompositeLateCancellationPosture, CompositePublicationIntent, OwnerExecutionOutcome,
    RuntimeWorldCancellationSource, RuntimeWorldPublicationOutcome,
};
use crate::recovery::ProductUnpublishedCause;

type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> crate::lifecycle::RuntimeWorldInstant {
        crate::lifecycle::RuntimeWorldInstant::from_ticks(7)
    }
}

fn budgets() -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 4,
            history_metadata_bytes: 16384,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 1,
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
            unique_exact_component_pins: 6,
            in_flight_pin_acquisition_reservations: 4,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 1,
        },
    })
    .expect("test budgets are positive")
}

fn setup() -> (
    RealReferenceFixture,
    Arc<TestOwner>,
    ProductBranchObservation,
) {
    let mut fixture = reference_test_fixture::real_fixture(8, 8);
    let owner = Arc::new(
        TestOwner::new(fixture.owner_inputs(budgets(), RuntimeWorldClock::from_source(FixedClock)))
            .expect("managed owner construction"),
    );
    let performed = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        crate::branch::RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        crate::branch::RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    (fixture, owner, performed.product_branch().clone())
}

/// One canonical Relational publication off the observed head. Lowering and
/// reservation are a single owner step, so no test can hold a lowered plan
/// without the capacity that authorizes executing it.
fn ready_relational(
    fixture: &RealReferenceFixture,
    owner: &TestOwner,
    expected: &ProductBranchObservation,
    operation_name: &str,
) -> crate::publication::CompositePublicationReady {
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner,
        expected.clone(),
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(operation_name),
            ),
        &cancellation.token(),
        None,
    )
    .expect("the observed head admits preparation and reserves its capacity");
    let execution_cancellation = RuntimeWorldCancellationSource::new();
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner,
        prepared,
        &execution_cancellation.token(),
    );
    let settlement = match outcome {
        OwnerExecutionOutcome::Settled(settlement) => settlement,
        other => panic!("the production owner execution must settle: {other:?}"),
    };
    let successor = settlement
        .successor_basis()
        .cloned()
        .expect("production owner execution returns its successor basis");
    settlement
        .ready(successor)
        .expect("the exact successor retention is available")
}

#[path = "publication_service_tests/outcomes.rs"]
mod outcomes;
#[path = "publication_service_tests/performed_recovery.rs"]
mod performed_recovery;
#[path = "publication_service_tests/retirement_race.rs"]
mod retirement_race;
