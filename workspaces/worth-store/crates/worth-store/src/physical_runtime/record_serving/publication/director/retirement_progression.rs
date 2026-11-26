//! The publication owner carries a retirement claim through its covering
//! checkpoint and completion. Serving consumes only the resulting outcome.

use worth_proof::TransitionOutcome;

use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{DisplacedArtifact, RetiredArtifact, RetirementClaim};
use crate::physical_runtime::{
    CompletedPhysicalCheckpoint, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointRequest, PhysicalCheckpointSubmission,
    PhysicalRetirementDenial,
};

pub(in crate::physical_runtime) enum BoundedRetirementStep {
    Retired(DisplacedArtifact),
    SkippedCompleted,
    Drained,
    BudgetExhausted,
    Denied(PhysicalRetirementDenial),
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn retire_displaced_artifact(
        &self,
        checkpoints: &PhysicalCheckpointSubmission,
    ) -> Result<Option<DisplacedArtifact>, PhysicalRetirementDenial> {
        match self.retire_displaced_artifact_step(checkpoints, |_| true) {
            BoundedRetirementStep::Retired(displaced) => Ok(Some(displaced)),
            BoundedRetirementStep::SkippedCompleted => Ok(None),
            BoundedRetirementStep::Drained => Err(PhysicalRetirementDenial::Absent),
            BoundedRetirementStep::Denied(denial) => Err(denial),
            BoundedRetirementStep::BudgetExhausted => {
                unreachable!("unbounded retirement accepts every artifact")
            }
        }
    }

    /// Runs one retirement step under `self.retirement_owner`.
    ///
    /// For a fresh claim, canonical checkpoint coverage (`CanonicalCoverageBeforePrune`)
    /// is established *before* any durable intent or `PendingRetirementRelease`
    /// is installed, so checkpoint failure reverts cleanly without wedging
    /// pending releases. For an already-installed `PendingRetirementRelease`,
    /// the claim is never reverted on retryable checkpoint failures.
    pub(in crate::physical_runtime) fn retire_displaced_artifact_step(
        &self,
        checkpoints: &PhysicalCheckpointSubmission,
        admits: impl Fn(u64) -> bool,
    ) -> BoundedRetirementStep {
        let Some(_owner) = self.try_begin_retirement() else {
            return BoundedRetirementStep::Denied(PhysicalRetirementDenial::Waiting);
        };
        if self.has_pending_extent_release() {
            if self
                .pending_extent_release_bytes()
                .is_some_and(|bytes| !admits(bytes))
            {
                return BoundedRetirementStep::BudgetExhausted;
            }
            let displaced = match self.commit_extent_release_intent() {
                Ok(displaced) => displaced,
                Err(denial) => return BoundedRetirementStep::Denied(denial),
            };
            let checkpoint = match covering_checkpoint(checkpoints, displaced.artifact) {
                Ok(completed) => completed,
                Err(denial) => return BoundedRetirementStep::Denied(denial),
            };
            let captured = checkpoint.basis().source().root().generation();
            if !crate::physical_runtime::durability::checkpoint_covers(&displaced, captured) {
                return BoundedRetirementStep::Denied(PhysicalRetirementDenial::Retained);
            }
            #[cfg(feature = "certification-test-authority")]
            self.pause_retirement_kill(1);
            return match self.finish_retirement(displaced, &checkpoint) {
                Ok(()) => BoundedRetirementStep::Retired(displaced),
                Err(denial) => BoundedRetirementStep::Denied(denial),
            };
        }
        let displaced = match self.root_owner.claim_retirement_within_budget(admits) {
            RetirementClaim::Claimed(displaced) => displaced,
            RetirementClaim::SkippedCompleted => return BoundedRetirementStep::SkippedCompleted,
            RetirementClaim::BudgetExhausted => return BoundedRetirementStep::BudgetExhausted,
            RetirementClaim::Absent => return BoundedRetirementStep::Drained,
            RetirementClaim::Denied(denial) => return BoundedRetirementStep::Denied(denial),
        };
        let checkpoint = match covering_checkpoint(checkpoints, displaced.artifact) {
            Ok(completed) => completed,
            Err(denial) => {
                self.revert_retirement_claim(displaced.artifact);
                return BoundedRetirementStep::Denied(denial);
            }
        };
        let captured = checkpoint.basis().source().root().generation();
        if !crate::physical_runtime::durability::checkpoint_covers(&displaced, captured) {
            self.revert_retirement_claim(displaced.artifact);
            return BoundedRetirementStep::Denied(PhysicalRetirementDenial::Retained);
        }
        if let Err(denial) = self.commit_fresh_retirement_intent(displaced) {
            return BoundedRetirementStep::Denied(denial);
        }
        #[cfg(feature = "certification-test-authority")]
        self.pause_retirement_kill(1);
        match self.finish_retirement(displaced, &checkpoint) {
            Ok(()) => BoundedRetirementStep::Retired(displaced),
            Err(denial) => BoundedRetirementStep::Denied(denial),
        }
    }
}

fn covering_checkpoint(
    checkpoints: &PhysicalCheckpointSubmission,
    artifact: RetiredArtifact,
) -> Result<CompletedPhysicalCheckpoint, PhysicalRetirementDenial> {
    use std::sync::atomic::{AtomicU64, Ordering};
    // Two retirements sharing a generation—and retries after a transient
    // checkpoint failure—must not replay a prior failed checkpoint outcome.
    static ATTEMPT: AtomicU64 = AtomicU64::new(1);
    let attempt = ATTEMPT.fetch_add(1, Ordering::Relaxed);
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&artifact.generation().to_le_bytes());
    key[8..16].copy_from_slice(&artifact.id().to_le_bytes());
    key[16] = artifact.action_code(false);
    key[24..32].copy_from_slice(&attempt.to_le_bytes());
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .expect("retirement checkpoint deadline is nonzero"),
    );
    let handle = match checkpoints.start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(
            crate::physical_runtime::PhysicalCheckpointStartFailure::Capture(ref kind),
        ) if checkpoint_scheduler_waiting(kind) => return Err(PhysicalRetirementDenial::Waiting),
        _ => return Err(PhysicalRetirementDenial::Checkpoint),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => Ok(completed),
        PhysicalCheckpointOutcome::ProvenNoEffect(no_effect) => match no_effect.cause() {
            crate::physical_runtime::PhysicalCheckpointProvenNoEffectCause::DeniedBeforeCandidate(
                ref kind,
            )
            | crate::physical_runtime::PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(
                ref kind,
            ) if checkpoint_scheduler_waiting(kind) => Err(PhysicalRetirementDenial::Waiting),
            _ => Err(PhysicalRetirementDenial::Checkpoint),
        },
        PhysicalCheckpointOutcome::Indeterminate(_) => Err(PhysicalRetirementDenial::Checkpoint),
    }
}

fn checkpoint_scheduler_waiting(
    kind: &crate::physical_runtime::PhysicalCheckpointCaptureFailureKind,
) -> bool {
    use crate::physical_runtime::PhysicalCheckpointCaptureFailureKind as Kind;
    matches!(
        kind,
        Kind::SchedulerCapacityUnavailable
            | Kind::BackgroundYielded
            | Kind::BackgroundDeferred
            | Kind::BackgroundThrottled
    )
}
