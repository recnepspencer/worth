//! Borrow one original request for an explicitly narrowed post-ack capture.
pub use std::num::NonZeroUsize;
pub use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
};
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryBootstrapAdvancementPhase,
    },
    application_installation::{
        WorthQueryApplicationPreparationDenial, WorthQueryCheckpointTransitionRecovery,
    },
};

pub fn stopped<Runtime>(
    mut transition: impl for<'scope> FnMut(
        &WorthQueryBootstrapAdvancementPhase<'scope>,
    )
        -> Result<Runtime, WorthQueryApplicationPreparationDenial>,
) -> Box<WorthQueryCheckpointTransitionRecovery> {
    let lease = lease(64 * 1024 * 1024);
    let result =
        with_bootstrap_advancement_from_request(ExecutionRequest::leased(&lease), |phase| {
            transition(&phase)
        })
        .expect("original bootstrap request opens");
    match result {
        Err(WorthQueryApplicationPreparationDenial::CheckpointTransitionCaptureStopped(
            pending,
        )) => pending,
        Err(cause) => panic!("capture-only limit must retain acknowledged custody: {cause:?}"),
        Ok(_) => panic!("capture-only limit must refuse native capture"),
    }
}

pub fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    bounded_lease(bytes, CancellationToken::new())
}

pub fn bounded_lease(
    bytes: u64,
    cancellation: CancellationToken,
) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(bytes), None),
        ),
        cancellation,
        deadline: None,
    })
    .unwrap()
}

/// Capture setup has its own original payer; the installed serving policy stays
/// unchanged. This is the same 64 MiB grant as this owner's transition fixture;
/// there is no pre-capture footprint quote.
pub fn capture<
    Schema: worth_query_decl::facade::application_schema::ApplicationSchema + 'static,
>(
    application: &worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
) -> worth_query_host::facade::application_installation::WorthQueryApplicationCheckpoint {
    let funding = lease(64 * 1024 * 1024);
    application.with_funded_host_advancement(
        worth_execution::ExecutionAllocationPolicy::Execution(&funding),
        |phase| application.capture_application_checkpoint(
            &phase,
            worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        ),
    ).expect("the original capture setup request opens")
     .expect("the complete original capture setup is funded")
}
