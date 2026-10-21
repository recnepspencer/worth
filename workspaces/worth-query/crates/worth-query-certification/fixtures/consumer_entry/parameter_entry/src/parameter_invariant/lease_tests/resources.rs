use worth_query_host::facade::runtime::RuntimeWorldBranchBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldCustodyBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldHistoryBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldObservationBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldPublicationBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldRecoveryBudgetInstallation;
use worth_query_host::facade::runtime::RuntimeWorldRetentionBudgetInstallation;
use worth_query_host::facade::runtime::WorthQueryInvalidationResources;
use worth_query_host::facade::runtime::WorthQueryProductWorldClock;
use worth_query_host::facade::runtime::WorthQueryProductWorldResources;

pub(super) fn world_resources() -> WorthQueryProductWorldResources {
    // Bounds this fixture World's serial request memory and deterministic work.
    let execution_policy = policy();
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 8,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 64,
                history_metadata_bytes: 1_048_576,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 32,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 8,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 8,
                retained_partial_metadata_bytes: 1_048_576,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 128,
                in_flight_pin_acquisition_reservations: 32,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 32,
            },
        },
        WorthQueryProductWorldClock::start(),
        WorthQueryInvalidationResources::new(),
        execution_policy,
    )
    .expect("the consumer's finite World resources are valid")
}

pub(super) fn policy() -> worth_foundational::ExecutionRequestPolicy {
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
