use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};
use crate::lifecycle::{RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldInstant};

type TestOwner = super::super::RuntimeWorldOwnerRoot<(), (), (), (), ()>;

struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(7)
    }
}

fn budgets(metadata_bytes: u64) -> RuntimeWorldBudgets {
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
            retained_product_unpublished_records: 2,
            retained_partial_metadata_bytes: metadata_bytes,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 6,
            in_flight_pin_acquisition_reservations: 4,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 1,
        },
    })
    .expect("recovery metadata test budgets")
}

#[test]
fn owner_constructor_installs_the_configured_recovery_metadata_ceiling() {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(4, 4);
    let owner = TestOwner::new(
        fixture.owner_inputs(budgets(37), RuntimeWorldClock::from_source(FixedClock)),
    )
    .expect("managed owner construction");
    assert!(
        matches!(
            owner
                .state
                .recovery
                .reserve_product_unpublished(owner.owner_identity()),
            Err(crate::recovery::RecoveryCatalogDenial::CapacityExhausted { maximum: Some(2) })
        ),
        "the actual metadata reservation must refuse before any effect"
    );
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
}
