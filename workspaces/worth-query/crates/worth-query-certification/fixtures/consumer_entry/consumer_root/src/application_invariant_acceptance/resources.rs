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

pub(super) const ACTIVE_WORLD_OBSERVATIONS: u64 = 32;

// Bounds this fixture World's serial request memory and deterministic work.
pub(super) const EXECUTION_POLICY: worth_foundational::ExecutionRequestPolicy =
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    );

pub(super) fn world_resources() -> WorthQueryProductWorldResources {
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
                active_observations: ACTIVE_WORLD_OBSERVATIONS,
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
        EXECUTION_POLICY,
    )
    .expect("the consumer's finite World resources are valid")
}

pub(super) fn application_limits(
) -> worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits {
    worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits::new(
        world_resources(),
        worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(32).expect("concurrent graph work is non-zero"),
        ),
        worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
