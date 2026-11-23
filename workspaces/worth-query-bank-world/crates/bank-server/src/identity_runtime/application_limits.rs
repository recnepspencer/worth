//! Bank's finite application limits: the bounded serial preset with Bank's
//! World population ceilings, finite execution window and query concurrency.
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_installation::WorthQueryApplicationLimits,
    primary_graph::SignalConditionalEvaluationBudget,
    runtime::{
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldCustodyBudgetInstallation,
        RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
        RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
        RuntimeWorldRetentionBudgetInstallation, WorthQueryApplicationQueryResourceProfile,
    },
};

// Twice the 64 MiB preparation ceiling leaves framework memory headroom.
// The first authenticated read after Directory reopen also verifies the recorded
// index checkpoint and its projections. The full Bank journey exceeds 16M there;
// the finite 128M window covers that recovery and ordinary advancement alike.
// Initial construction admits the whole supplied source graph at once under
// the same finite window, as its own bootstrap grant.
pub(crate) const BANK_EXECUTION_POLICY: ExecutionRequestPolicy = ExecutionRequestPolicy::new(
    ExecutionPosture::Serial,
    DeterminismContract::CanonicalBitwise,
    ExecutionBudget::new(
        std::num::NonZeroUsize::MIN,
        Some(128 * 1024 * 1024),
        Some(128_000_000),
    ),
);

/// Bank's installed limits under `execution`, ordinarily [`BANK_EXECUTION_POLICY`].
pub(crate) fn bank_application_limits(
    execution: ExecutionRequestPolicy,
) -> WorthQueryApplicationLimits {
    WorthQueryApplicationLimits::bounded_serial(WorthQueryAdmissionClock::system())
        .with_branches(RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 1_024,
        })
        .with_history(RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 8_192,
            history_metadata_bytes: 128 * 1024 * 1024,
        })
        .with_observations(RuntimeWorldObservationBudgetInstallation {
            active_observations: 4_096,
        })
        .with_publication(RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 1_024,
        })
        .with_recovery(RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 1_024,
            retained_partial_metadata_bytes: 128 * 1024 * 1024,
        })
        .with_retention(RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 8_192,
            in_flight_pin_acquisition_reservations: 2_048,
        })
        .with_custody(RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 2_048,
        })
        .with_execution_policy(execution)
        .with_query_profile(WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(64).expect("native graph concurrency is nonzero"),
        ))
        .with_signal_budget(SignalConditionalEvaluationBudget::development())
        .install()
        .expect("the bank application limits are statically valid")
}
