//! Bounded dirty-generation streaming into one checkpoint candidate.

use worth_store_buffer_pool::{
    PhysicalDirtyGenerationCaptureCompletion, PhysicalDirtyGenerationCaptureSession,
    PhysicalDirtyGenerationCaptureStep, PhysicalDirtyGenerationSlice,
};

use super::execution::{
    append_slice, remove_created_after_failure, remove_created_candidate,
    PhysicalCheckpointExecutionResult,
};
use super::PhysicalCheckpointCaptureOwner;
use crate::physical_runtime::durability::checkpoint::publication::CreatedCheckpointCandidate;
use crate::physical_runtime::durability::checkpoint::{
    PhysicalCheckpointAttempt, PhysicalCheckpointCaptureFailureKind,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointProgressPhase,
    PhysicalCheckpointProvenNoEffectCause,
};

enum DirtyCaptureContinuation {
    More(PhysicalDirtyGenerationCaptureSession),
    Complete(PhysicalDirtyGenerationCaptureCompletion),
}

impl PhysicalCheckpointCaptureOwner {
    pub(super) fn capture_dirty_generation(
        &self,
        mut session: PhysicalDirtyGenerationCaptureSession,
        mut candidate: CreatedCheckpointCandidate,
        attempt: &PhysicalCheckpointAttempt,
        key: PhysicalCheckpointIdempotencyKey,
    ) -> Result<
        (
            CreatedCheckpointCandidate,
            PhysicalDirtyGenerationCaptureCompletion,
        ),
        PhysicalCheckpointExecutionResult,
    > {
        attempt.enter(PhysicalCheckpointProgressPhase::Capture);
        loop {
            let step = match self.next_capture_step(session) {
                Ok(step) => step,
                Err(kind) => {
                    return Err(remove_created_candidate(
                        candidate,
                        attempt,
                        key,
                        PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(kind),
                    ));
                }
            };
            let (slice, continuation) = split_capture_step(step);
            if let Err(failure) = emit_slice(&mut candidate, &slice, attempt) {
                return Err(remove_created_after_failure(
                    candidate, attempt, key, failure,
                ));
            }
            match continuation {
                DirtyCaptureContinuation::More(next) => {
                    if attempt.cancellation_requested() {
                        return Err(remove_created_candidate(
                            candidate,
                            attempt,
                            key,
                            PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved,
                        ));
                    }
                    session = next;
                }
                DirtyCaptureContinuation::Complete(completion) => {
                    return Ok((candidate, completion));
                }
            }
        }
    }

    fn next_capture_step(
        &self,
        session: PhysicalDirtyGenerationCaptureSession,
    ) -> Result<PhysicalDirtyGenerationCaptureStep, PhysicalCheckpointCaptureFailureKind> {
        let allocation = self
            .frames
            .checkpoint_capture_window(&session, self.checkpoint_policy.memory_limit().get())
            .map_err(PhysicalCheckpointCaptureFailureKind::ResidencyAdmission)?;
        self.frames
            .capture_checkpoint_slice(session, allocation)
            .map_err(PhysicalCheckpointCaptureFailureKind::ResidencyAdmission)
    }
}

fn split_capture_step(
    step: PhysicalDirtyGenerationCaptureStep,
) -> (PhysicalDirtyGenerationSlice, DirtyCaptureContinuation) {
    match step {
        PhysicalDirtyGenerationCaptureStep::More { session, slice } => {
            (slice, DirtyCaptureContinuation::More(session))
        }
        PhysicalDirtyGenerationCaptureStep::Complete { completion, slice } => {
            (slice, DirtyCaptureContinuation::Complete(completion))
        }
    }
}

fn emit_slice(
    candidate: &mut CreatedCheckpointCandidate,
    slice: &PhysicalDirtyGenerationSlice,
    attempt: &PhysicalCheckpointAttempt,
) -> Result<(), crate::physical_runtime::durability::checkpoint::PhysicalCheckpointCaptureFailure> {
    let capture_allocation = attempt.begin_capture_allocation(slice.metadata_bytes());
    append_slice(candidate, slice)?;
    attempt.record_capture(candidate.dirty_records(), candidate.encoded_bytes());
    drop(capture_allocation);
    Ok(())
}
