use worth_query_host::facade::runtime::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    WorthQueryProductWorldClock, WorthQueryProductWorldResources,
};

pub(crate) fn bank_product_world_resources() -> WorthQueryProductWorldResources {
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 1_024,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 8_192,
                history_metadata_bytes: 128 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 4_096,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 1_024,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 1_024,
                retained_partial_metadata_bytes: 128 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 8_192,
                in_flight_pin_acquisition_reservations: 2_048,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 2_048,
            },
        },
        WorthQueryProductWorldClock::start(),
    )
    .expect("the bank Product World resources are statically valid")
}
