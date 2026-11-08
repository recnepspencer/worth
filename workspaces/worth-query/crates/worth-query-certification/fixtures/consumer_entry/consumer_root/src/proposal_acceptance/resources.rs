//! Normal finite installation profile for this small public application.
use worth_query_host::facade::{application_installation::WorthQueryApplicationLimits, runtime::*};

pub fn limits() -> WorthQueryApplicationLimits {
    let execution = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let world = WorthQueryProductWorldResources::install(
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
        WorthQueryProductWorldClock::start(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        WorthQueryInvalidationResources::new(),
        execution,
    )
    .unwrap();
    WorthQueryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(32).unwrap()),
        worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
