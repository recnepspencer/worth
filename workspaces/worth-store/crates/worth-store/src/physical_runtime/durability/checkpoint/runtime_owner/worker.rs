use super::*;

pub(super) fn run_worker(
    owner: Weak<PhysicalCheckpointRuntimeOwner>,
    attempt: Arc<PhysicalCheckpointAttempt>,
    admitted: super::super::capture::AdmittedPhysicalCheckpointCapture,
) {
    let Some(owner) = owner.upgrade() else {
        attempt.complete(super::super::PhysicalCheckpointOutcome::Indeterminate(
            super::super::IndeterminatePhysicalCheckpoint::new(
                attempt.identity(),
                attempt.idempotency_key(),
                PhysicalCheckpointCaptureFailureKind::RuntimeUnavailable,
            ),
        ));
        return;
    };
    let execution = catch_unwind(AssertUnwindSafe(|| {
        owner.capture.execute(admitted, &attempt)
    }));
    let (result, panicked) = match execution {
        Ok(result) => (result, false),
        Err(_) => (
            PhysicalCheckpointExecutionResult::indeterminate(
                attempt.identity(),
                attempt.idempotency_key(),
                PhysicalCheckpointCaptureFailureKind::WorkerPanicked,
            ),
            true,
        ),
    };
    let terminal = result.terminal();
    owner.record_result(result, panicked);
    owner.capture.release_background_selection();
    attempt.complete(terminal);
}
