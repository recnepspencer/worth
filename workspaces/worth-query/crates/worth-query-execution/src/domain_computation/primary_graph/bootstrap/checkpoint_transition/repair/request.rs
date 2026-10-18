//! Move-only recovery survives failure to enter the caller's original request.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::with_bootstrap_advancement_from_request;
impl WorthQueryCheckpointTransitionRecovery {
    /// Repair the same performed transition under original borrowed execution.
    /// Opening refusal returns this capsule; no captured closure can drop it.
    pub fn repair_to_checkpoint_with_request(
        self,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationCheckpoint, Self> {
        let mut pending = Some(self);
        match with_bootstrap_advancement_from_request(request, |phase| {
            let mut original = pending
                .take()
                .expect("entry consumes original repair custody once");
            let policy = match phase.funded_capture_policy() {
                Ok(policy) => policy,
                Err(cause) => {
                    original
                        .retain_capture_denial(WorthQueryCheckpointCaptureDenial::Execution(cause));
                    return Err(original);
                }
            };
            original.repair_to_checkpoint(&phase, policy)
        }) {
            Ok(result) => result,
            Err(cause) => {
                let mut original = pending.expect("opening refusal did not enter repair");
                original.retain_capture_denial(WorthQueryCheckpointCaptureDenial::Execution(
                    super::super::super::request_denial(cause),
                ));
                Err(original)
            }
        }
    }
}
