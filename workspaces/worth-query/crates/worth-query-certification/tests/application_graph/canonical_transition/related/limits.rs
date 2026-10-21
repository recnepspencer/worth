//! Finite host execution policy, consumed by ordinary installation and requests.
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationLimits;
use worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget;
use worth_query_host::facade::runtime::*;

pub fn limits(work: u64) -> WorthQueryInMemoryApplicationLimits {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(64 * 1024 * 1024),
            Some(work),
        ),
    );
    let resources = WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 8,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 64,
                history_metadata_bytes: 4 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 32,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 8,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 8,
                retained_partial_metadata_bytes: 4 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 64,
                in_flight_pin_acquisition_reservations: 16,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 32,
            },
        },
        WorthQueryProductWorldClock::start(),
        WorthQueryInvalidationResources::new(),
        policy,
    )
    .unwrap();
    WorthQueryInMemoryApplicationLimits::new(
        resources,
        WorthQueryApplicationCandidateResourceProfile::policy(Some(1024), Some(1024 * 1024))
            .unwrap(),
        WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(64).expect("concurrent graph work is non-zero"),
        ),
        SignalConditionalEvaluationBudget::development(),
    )
}
