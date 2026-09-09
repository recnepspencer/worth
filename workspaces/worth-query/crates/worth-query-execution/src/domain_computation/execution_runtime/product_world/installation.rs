use std::time::Instant;

use worth_runtime_world::facade::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation, RuntimeWorldBudgets,
    RuntimeWorldClockSource, RuntimeWorldCustodyBudgetInstallation,
    RuntimeWorldHistoryBudgetInstallation, RuntimeWorldInstant,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};

#[derive(Clone)]
pub(crate) struct WorthQueryProductWorldClock {
    origin: Instant,
}

impl WorthQueryProductWorldClock {
    pub(crate) fn start() -> Self {
        Self {
            origin: Instant::now(),
        }
    }

    pub(crate) fn deadline(&self, deadline: Instant) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(
            deadline
                .saturating_duration_since(self.origin)
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64,
        )
    }
}

impl RuntimeWorldClockSource for WorthQueryProductWorldClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(
            self.origin.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
        )
    }
}

pub(crate) fn installed_budgets() -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 128,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 1_024,
            history_metadata_bytes: 16 * 1024 * 1024,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 512,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 128,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 128,
            retained_partial_metadata_bytes: 16 * 1024 * 1024,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 1_024,
            in_flight_pin_acquisition_reservations: 256,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 256,
        },
    })
    .expect("Query's installed Runtime World budget is valid")
}
