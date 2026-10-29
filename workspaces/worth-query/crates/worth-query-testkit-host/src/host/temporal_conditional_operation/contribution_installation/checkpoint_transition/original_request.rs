//! Actual native installation borrows original request work, controls and capture funding.
pub use super::*;
pub use application_installation::WorthQueryInMemoryApplicationDenial as InstallationDenial;
pub use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    runtime::{CancellationSource, CancellationToken, LeaseRequest},
};

pub fn request(cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::new(1).unwrap(),
                Some(128 * 1024 * 1024),
                Some(16_000_000),
            ),
        ),
        deadline: None,
        cancellation,
    }
}
pub type Target = application_installation::WorthQueryProgramApplicationRuntime<
    TemporalHostSchema,
    TemporalInstallationProgram,
>;
pub fn transition_original(
    original: ExecutionRequest<'_, '_>,
    checkpoint: application_installation::WorthQueryApplicationCheckpoint,
    predecessor: Predecessor,
    author: impl FnOnce(
        &mut application_installation::WorthQueryCheckpointMigrationWriter<'_, TemporalHostSchema>,
        &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<Target, InstallationDenial> {
    with_bootstrap_advancement_from_request(original, |phase| {
        application_installation::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement(
            &phase, validated_program(), application_installation::WorthQueryApplicationProgramRoster::new(),
            TemporalHostSchema::declaration().unwrap(), configuration(), ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(), checkpoint::checkpoint_limits(),
            checkpoint, predecessor, Resources::bounded(512, 32, 4096).unwrap(),
            // The original request's own lease caps the capture it funds.
            phase.execution_request().allocation_policy(), author)
    }).expect("the original native request enters without another Query opening")
}
pub fn assert_cancelled(cause: &primary_graph::WorthQueryPrimaryGraphInstallationDenial) {
    assert!(matches!(cause.kind(), primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
        kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled
    }), "{cause:?}");
}
pub fn assert_work(cause: &primary_graph::WorthQueryPrimaryGraphInstallationDenial) {
    assert!(matches!(cause.kind(), primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
        kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted, .. }
    }), "work exhaustion, not nested opening or unrelated lease: {cause:?}");
}
