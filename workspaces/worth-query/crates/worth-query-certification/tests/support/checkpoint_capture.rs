//! Borrow one original request for an explicitly narrowed post-ack capture.
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryBootstrapAdvancementPhase,
    },
    application_installation::{
        WorthQueryCheckpointTransitionRecovery, WorthQueryInMemoryApplicationDenial,
    },
};

pub fn stopped<Runtime>(
    mut transition: impl for<'scope> FnMut(
        &WorthQueryBootstrapAdvancementPhase<'scope>,
    ) -> Result<Runtime, WorthQueryInMemoryApplicationDenial>,
) -> Box<WorthQueryCheckpointTransitionRecovery> {
    let lease = lease(64 * 1024 * 1024);
    let result =
        with_bootstrap_advancement_from_request(ExecutionRequest::leased(&lease), |phase| {
            transition(&phase)
        })
        .expect("original bootstrap request opens");
    match result {
        Err(WorthQueryInMemoryApplicationDenial::CheckpointTransitionCaptureStopped(pending)) => {
            pending
        }
        Err(cause) => panic!("capture-only limit must retain acknowledged custody: {cause:?}"),
        Ok(_) => panic!("capture-only limit must refuse native capture"),
    }
}

pub fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    bounded_lease(bytes, CancellationToken::new())
}

fn bounded_lease(bytes: u64, cancellation: CancellationToken) -> ExecutionResourceLease<'static> {
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
