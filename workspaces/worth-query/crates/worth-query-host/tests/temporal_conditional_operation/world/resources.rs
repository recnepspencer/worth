use worth_query_host::facade::runtime::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    WorthQueryProductWorldClock, WorthQueryProductWorldResources,
};

impl super::CourtroomWorld {
    #[allow(dead_code)] // Shared world compiled by independent test targets.
    pub fn publish_with_result_bytes(gate: &str, bytes: usize) -> Self {
        let contacts = super::ContactCounters::default();
        let (predicate, panic) = super::Predicate::controlled(contacts.clone());
        Self::publish_with_predicate(
            gate,
            0,
            contacts,
            predicate,
            panic,
            None,
            None,
            None,
            1,
            false,
            Some(bytes),
        )
    }
}

pub(crate) fn product_world_resources(
    retained_composite_commits: u64,
) -> WorthQueryProductWorldResources {
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 128,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits,
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
        },
        WorthQueryProductWorldClock::start(),
        worth_query_execution::facade::runtime::WorthQueryInvalidationResources::install(
            worth_query_execution::facade::runtime::WorthQueryInvalidationResourceInstallation::bounded(
                1_000_000,
                64 * 1_024 * 1_024,
                128 * 1_024 * 1_024,
                128,
            ),
        )
        .expect("the Query invalidation installation is valid"),
    )
    .expect("the courtroom Product World resources are valid")
}
