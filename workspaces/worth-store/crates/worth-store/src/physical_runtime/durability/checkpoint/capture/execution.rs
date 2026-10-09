//! Effectful checkpoint capture progression and candidate reconciliation.

use worth_store_buffer_pool::{
    PhysicalDirtyGenerationCaptureCompletion, PhysicalDirtyGenerationSlice,
};
use worth_store_physical_format::{CheckpointDirtyFrameBasis, PhysicalCheckpointIdentity};

use super::{AdmittedPhysicalCheckpointCapture, PhysicalCheckpointCaptureOwner};
use crate::physical_runtime::durability::checkpoint::publication::{
    CapturedCheckpointCandidate, CheckpointCandidateCleanup, CreatedCheckpointCandidate,
    DurableCheckpointCandidate,
};
use crate::physical_runtime::durability::checkpoint::{
    IndeterminatePhysicalCheckpoint, PhysicalCheckpointAttempt, PhysicalCheckpointCaptureFailure,
    PhysicalCheckpointCaptureFailureKind, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointProgressPhase,
    PhysicalCheckpointProvenNoEffectCause, PhysicalCheckpointPublication,
    ProvenNoEffectPhysicalCheckpoint,
};

pub(in crate::physical_runtime) struct PhysicalCheckpointExecutionResult {
    terminal: PhysicalCheckpointOutcome,
    publication: Option<PhysicalCheckpointPublication>,
}

impl PhysicalCheckpointCaptureOwner {
    pub(in crate::physical_runtime) fn execute(
        &self,
        admitted: AdmittedPhysicalCheckpointCapture,
        attempt: &PhysicalCheckpointAttempt,
    ) -> PhysicalCheckpointExecutionResult {
        let basis = admitted.basis;
        let custody = admitted.custody;
        let key = attempt.idempotency_key();
        if attempt.cancellation_requested() {
            return PhysicalCheckpointExecutionResult::no_effect(
                basis.identity(),
                key,
                PhysicalCheckpointProvenNoEffectCause::CancelledBeforeCandidate,
            );
        }
        let candidate = match self.create_capture_candidate(basis, custody, attempt, key) {
            Ok(candidate) => candidate,
            Err(terminal) => return terminal,
        };
        let (candidate, completion) =
            match self.capture_dirty_generation(admitted.session, candidate, attempt, key) {
                Ok(captured) => captured,
                Err(terminal) => return terminal,
            };
        if !self.capture_completion_matches_basis(&completion, basis) {
            return remove_created_candidate(
                candidate,
                attempt,
                key,
                PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(
                    PhysicalCheckpointCaptureFailureKind::SourceAuthorityMismatch,
                ),
            );
        }
        self.finalize_and_publish_candidate(candidate, attempt, basis, key)
    }

    fn create_capture_candidate(
        &self,
        basis: super::PhysicalCheckpointCaptureBasis,
        custody: Option<crate::physical_runtime::durability::SelectedCheckpointCustodySnapshot>,
        attempt: &PhysicalCheckpointAttempt,
        key: PhysicalCheckpointIdempotencyKey,
    ) -> Result<CreatedCheckpointCandidate, PhysicalCheckpointExecutionResult> {
        attempt.enter(PhysicalCheckpointProgressPhase::CandidateCreation);
        let candidate = CreatedCheckpointCandidate::create(basis, self.work.clone(), custody)
            .map_err(|(cleanup, action)| {
                initial_action_failure(cleanup, attempt, basis.identity(), key, action)
            })?;
        if attempt.cancellation_requested() {
            return Err(remove_created_candidate(
                candidate,
                attempt,
                key,
                PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved,
            ));
        }
        Ok(candidate)
    }

    fn capture_completion_matches_basis(
        &self,
        completion: &PhysicalDirtyGenerationCaptureCompletion,
        basis: super::PhysicalCheckpointCaptureBasis,
    ) -> bool {
        completion.store_identity() == self.store
            && completion.frontier().get() == basis.source().dirty_generation_frontier()
    }
}

pub(super) fn finish_capture_candidate(
    candidate: CreatedCheckpointCandidate,
    binding_compaction: &crate::physical_runtime::durability::PhysicalMutationBindingCompactionCutover<'_>,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
) -> Result<CapturedCheckpointCandidate, PhysicalCheckpointExecutionResult> {
    if attempt.cancellation_requested() {
        return Err(remove_created_candidate(
            candidate,
            attempt,
            key,
            PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved,
        ));
    }
    candidate
        .finish(binding_compaction)
        .map_err(|(cleanup, failure)| remove_cleanup_after_failure(cleanup, attempt, key, failure))
}

pub(super) fn synchronize_capture_candidate(
    captured: CapturedCheckpointCandidate,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
) -> Result<DurableCheckpointCandidate, PhysicalCheckpointExecutionResult> {
    if attempt.cancellation_requested() {
        return Err(remove_captured_candidate(
            captured,
            attempt,
            key,
            PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved,
        ));
    }
    attempt.enter(PhysicalCheckpointProgressPhase::CandidateSynchronization);
    captured
        .synchronize()
        .map_err(|(cleanup, failure)| remove_cleanup_after_failure(cleanup, attempt, key, failure))
}

impl PhysicalCheckpointExecutionResult {
    pub(super) fn completed(publication: PhysicalCheckpointPublication) -> Self {
        Self {
            terminal: PhysicalCheckpointOutcome::Completed(publication.completed_observation()),
            publication: Some(publication),
        }
    }

    fn no_effect(
        identity: PhysicalCheckpointIdentity,
        key: PhysicalCheckpointIdempotencyKey,
        cause: PhysicalCheckpointProvenNoEffectCause,
    ) -> Self {
        Self {
            terminal: PhysicalCheckpointOutcome::ProvenNoEffect(
                ProvenNoEffectPhysicalCheckpoint::new(identity, key, cause),
            ),
            publication: None,
        }
    }

    pub(in crate::physical_runtime) fn indeterminate(
        identity: PhysicalCheckpointIdentity,
        key: PhysicalCheckpointIdempotencyKey,
        failure: PhysicalCheckpointCaptureFailureKind,
    ) -> Self {
        Self {
            terminal: PhysicalCheckpointOutcome::Indeterminate(
                IndeterminatePhysicalCheckpoint::new(identity, key, failure),
            ),
            publication: None,
        }
    }

    pub(in crate::physical_runtime) fn terminal(&self) -> PhysicalCheckpointOutcome {
        self.terminal.clone()
    }

    pub(in crate::physical_runtime) fn into_publication(
        self,
    ) -> Option<PhysicalCheckpointPublication> {
        self.publication
    }
}

pub(super) fn append_slice(
    candidate: &mut CreatedCheckpointCandidate,
    slice: &PhysicalDirtyGenerationSlice,
) -> Result<(), PhysicalCheckpointCaptureFailure> {
    for frame in slice.frames() {
        if frame.frame().store() != candidate.basis().identity().store_identity() {
            return Err(
                PhysicalCheckpointCaptureFailure::candidate_requires_inspection(
                    PhysicalCheckpointCaptureFailureKind::SourceAuthorityMismatch,
                ),
            );
        }
        candidate
            .append_dirty(CheckpointDirtyFrameBasis::new(
                frame.frame().coordinate(),
                frame.generation().get(),
            ))
            .map_err(PhysicalCheckpointCaptureFailure::from_initial_action)?;
    }
    Ok(())
}

fn initial_action_failure(
    cleanup: CheckpointCandidateCleanup,
    attempt: &PhysicalCheckpointAttempt,
    identity: PhysicalCheckpointIdentity,
    key: PhysicalCheckpointIdempotencyKey,
    action: crate::physical_runtime::durability::checkpoint::PhysicalCheckpointActionFailure,
) -> PhysicalCheckpointExecutionResult {
    let failure = PhysicalCheckpointCaptureFailure::from_initial_action(action);
    if failure.requires_inspection() {
        remove_cleanup_after_capture_failure(cleanup, attempt, key, failure)
    } else {
        PhysicalCheckpointExecutionResult::no_effect(
            identity,
            key,
            PhysicalCheckpointProvenNoEffectCause::DeniedBeforeCandidate(failure.kind()),
        )
    }
}

pub(super) fn remove_created_after_failure(
    candidate: CreatedCheckpointCandidate,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
    failure: PhysicalCheckpointCaptureFailure,
) -> PhysicalCheckpointExecutionResult {
    remove_created_candidate(
        candidate,
        attempt,
        key,
        PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(failure.kind()),
    )
}

pub(super) fn remove_created_candidate(
    candidate: CreatedCheckpointCandidate,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
    cause: PhysicalCheckpointProvenNoEffectCause,
) -> PhysicalCheckpointExecutionResult {
    attempt.enter(PhysicalCheckpointProgressPhase::CandidateCleanup);
    let identity = candidate.basis().identity();
    match candidate.remove() {
        Ok(_cleanup) => PhysicalCheckpointExecutionResult::no_effect(identity, key, cause),
        Err(_failure) => PhysicalCheckpointExecutionResult::indeterminate(
            identity,
            key,
            PhysicalCheckpointCaptureFailureKind::CandidateContinuationFailed,
        ),
    }
}

fn remove_captured_candidate(
    candidate: CapturedCheckpointCandidate,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
    cause: PhysicalCheckpointProvenNoEffectCause,
) -> PhysicalCheckpointExecutionResult {
    attempt.enter(PhysicalCheckpointProgressPhase::CandidateCleanup);
    let identity = candidate.basis().identity();
    match candidate.remove() {
        Ok(_cleanup) => PhysicalCheckpointExecutionResult::no_effect(identity, key, cause),
        Err(_failure) => PhysicalCheckpointExecutionResult::indeterminate(
            identity,
            key,
            PhysicalCheckpointCaptureFailureKind::CandidateContinuationFailed,
        ),
    }
}

pub(super) fn remove_durable_candidate(
    candidate: DurableCheckpointCandidate,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
    cause: PhysicalCheckpointProvenNoEffectCause,
) -> PhysicalCheckpointExecutionResult {
    attempt.enter(PhysicalCheckpointProgressPhase::CandidateCleanup);
    let identity = candidate.basis().identity();
    match candidate.remove() {
        Ok(_cleanup) => PhysicalCheckpointExecutionResult::no_effect(identity, key, cause),
        Err(_failure) => PhysicalCheckpointExecutionResult::indeterminate(
            identity,
            key,
            PhysicalCheckpointCaptureFailureKind::CandidateContinuationFailed,
        ),
    }
}

fn remove_cleanup_after_failure(
    cleanup: CheckpointCandidateCleanup,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
    failure: crate::physical_runtime::durability::checkpoint::PhysicalCheckpointActionFailure,
) -> PhysicalCheckpointExecutionResult {
    let capture = PhysicalCheckpointCaptureFailure::from_initial_action(failure);
    remove_cleanup_after_capture_failure(cleanup, attempt, key, capture)
}

fn remove_cleanup_after_capture_failure(
    cleanup: CheckpointCandidateCleanup,
    attempt: &PhysicalCheckpointAttempt,
    key: PhysicalCheckpointIdempotencyKey,
    capture: PhysicalCheckpointCaptureFailure,
) -> PhysicalCheckpointExecutionResult {
    attempt.enter(PhysicalCheckpointProgressPhase::CandidateCleanup);
    let identity = cleanup.identity();
    match cleanup.remove() {
        Ok(_cleanup) => PhysicalCheckpointExecutionResult::no_effect(
            identity,
            key,
            PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(capture.kind()),
        ),
        Err(_failure) => PhysicalCheckpointExecutionResult::indeterminate(
            identity,
            key,
            PhysicalCheckpointCaptureFailureKind::CandidateContinuationFailed,
        ),
    }
}
