pub use worth_query_host::facade::runtime::RuntimeWorldBranchBudgetInstallation;
pub use worth_query_host::facade::runtime::RuntimeWorldBudgetInstallation;
pub use worth_query_host::facade::runtime::RuntimeWorldCustodyBudgetInstallation;
pub use worth_query_host::facade::runtime::RuntimeWorldHistoryBudgetInstallation;
pub use worth_query_host::facade::runtime::RuntimeWorldObservationBudgetInstallation;
pub use worth_query_host::facade::runtime::RuntimeWorldPublicationBudgetInstallation;
pub use worth_query_host::facade::runtime::RuntimeWorldRecoveryBudgetInstallation;
pub use worth_query_host::facade::runtime::RuntimeWorldRetentionBudgetInstallation;
pub use worth_query_host::facade::runtime::WorthQueryProductWorldClock;
pub use worth_query_host::facade::runtime::WorthQueryProductWorldResources;

pub fn product_world_resources(retained_composite_commits: u64) -> WorthQueryProductWorldResources {
    let execution_policy = execution_policy();
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
        worth_query_execution::facade::runtime::WorthQueryInvalidationResources::new(),
        execution_policy,
    )
    .expect("the courtroom Product World resources are valid")
}

/// The named policy shared by installation and every executing fixture door.
pub fn execution_policy() -> worth_foundational::ExecutionRequestPolicy {
    // Bounds this fixture World's serial request memory and deterministic work.
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    )
}
