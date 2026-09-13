use worth_runtime_world::facade::*;
pub fn court() -> RuntimeWorldBudgets {
    limited(512, 1024)
}
pub fn limited(history: u64, pins: u64) -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 128,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: history,
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
            unique_exact_component_pins: pins,
            // Each prepared attempt reserves two exact-owner acquisitions.
            in_flight_pin_acquisition_reservations: 256,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 256,
        },
    })
    .unwrap()
}
pub struct CourtClock;
impl RuntimeWorldClockSource for CourtClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(100)
    }
}
