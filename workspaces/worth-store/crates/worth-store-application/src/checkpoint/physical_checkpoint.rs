//! Bound physical WAL replay after the single home manifest is selected.
use super::ApplicationHomeCheckpointDenial;

pub(super) fn complete(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    generation: u64,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeCheckpointDenial> {
    use worth_proof::TransitionOutcome;
    use worth_store::physical_runtime::{
        PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
        PhysicalCheckpointRequest,
    };
    let failure = |detail: String| ApplicationHomeCheckpointDenial::PhysicalCheckpoint { detail };
    let mut key = [0_u8; 32];
    key[..8].copy_from_slice(b"WAPHCKP1");
    key[8..16].copy_from_slice(&generation.to_le_bytes());
    let milliseconds = crate::deadline::milliseconds(execution, 120_000).map_err(|cause| {
        ApplicationHomeCheckpointDenial::PhysicalCheckpointRequestStopped { cause }
    })?;
    let deadline = PhysicalCheckpointDeadline::after_milliseconds(milliseconds)
        .ok_or_else(|| failure("checkpoint deadline is not representable".to_owned()))?;
    let request =
        PhysicalCheckpointRequest::fuzzy(PhysicalCheckpointIdempotencyKey::new(key), deadline);
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        other => {
            return Err(failure(format!(
                "physical checkpoint start {}",
                crate::open::physical_admission::unfinished_outcome(&other)
            )))
        }
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => Ok(()),
        other => Err(failure(format!("{other:?}"))),
    }
}
