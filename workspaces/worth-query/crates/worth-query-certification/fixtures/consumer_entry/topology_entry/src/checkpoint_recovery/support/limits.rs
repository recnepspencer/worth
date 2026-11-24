//! Application resource limits for topology_entry checkpoint recovery courts.
use super::*;
use worth_query_host::facade::runtime::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    WorthQueryApplicationQueryResourceProfile, WorthQueryProductWorldClock,
    WorthQueryProductWorldResources,
};

pub(in crate::checkpoint_recovery) fn invalidation(
) -> worth_query_host::facade::runtime::WorthQueryInvalidationResources {
    worth_query_host::facade::runtime::WorthQueryInvalidationResources::new()
}

pub(in crate::checkpoint_recovery) fn limits(
    retained_composite_commits: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
) -> WorthQueryApplicationLimits {
    limits_with_room(retained_composite_commits, 16, 64, invalidation)
}

/// A world with room for a long history under open demands. Rows settled at
/// different commits keep their own product observations active, and every
/// open demand pins the exact components it settled on.
pub(in crate::checkpoint_recovery) fn limits_with_room(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
) -> WorthQueryApplicationLimits {
    limits_with_policy(
        retained_composite_commits,
        active_observations,
        unique_exact_component_pins,
        invalidation,
        execution_policy::CHECKPOINT_EXECUTION_POLICY,
    )
}

pub(in crate::checkpoint_recovery) fn limits_with_policy(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
    policy: worth_foundational::ExecutionRequestPolicy,
) -> WorthQueryApplicationLimits {
    limits_with_history_and_policy(
        retained_composite_commits,
        active_observations,
        unique_exact_component_pins,
        invalidation,
        524_288,
        policy,
        authentication::WorthQueryAdmissionClock::system(),
    )
}

pub(in crate::checkpoint_recovery) fn limits_with_history_and_policy(
    retained_composite_commits: u64,
    active_observations: u64,
    unique_exact_component_pins: u64,
    invalidation: worth_query_host::facade::runtime::WorthQueryInvalidationResources,
    history_metadata_bytes: u64,
    policy: worth_foundational::ExecutionRequestPolicy,
    clock: authentication::WorthQueryAdmissionClock,
) -> WorthQueryApplicationLimits {
    WorthQueryApplicationLimits::new(
        WorthQueryProductWorldResources::install(
            RuntimeWorldBudgetInstallation {
                branches: RuntimeWorldBranchBudgetInstallation {
                    live_product_branches: 4,
                },
                history: RuntimeWorldHistoryBudgetInstallation {
                    retained_composite_commits,
                    history_metadata_bytes,
                },
                observations: RuntimeWorldObservationBudgetInstallation {
                    active_observations,
                },
                publication: RuntimeWorldPublicationBudgetInstallation {
                    active_publication_attempts: 4,
                },
                recovery: RuntimeWorldRecoveryBudgetInstallation {
                    retained_product_unpublished_records: 4,
                    retained_partial_metadata_bytes: 524_288,
                },
                retention: RuntimeWorldRetentionBudgetInstallation {
                    unique_exact_component_pins,
                    in_flight_pin_acquisition_reservations: 16,
                },
                custody: RuntimeWorldCustodyBudgetInstallation {
                    owner_created_component_custody_records: 16,
                },
            },
            WorthQueryProductWorldClock::start(clock),
            invalidation,
            policy,
        )
        .unwrap(),
        WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(32).expect("concurrent graph work is non-zero"),
        ),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
