//! The publication owner carries a retirement claim through its covering
//! checkpoint and completion. Serving consumes only the resulting outcome.

use worth_proof::TransitionOutcome;

use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{DisplacedArtifact, RetiredArtifact};
use crate::physical_runtime::{
    CompletedPhysicalCheckpoint, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointRequest, PhysicalCheckpointSubmission,
    PhysicalRetirementDenial,
};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn retire_displaced_artifact(
        &self,
        checkpoints: &PhysicalCheckpointSubmission,
    ) -> Result<Option<DisplacedArtifact>, PhysicalRetirementDenial> {
        let Some(_owner) = self.try_begin_retirement() else {
            return Err(PhysicalRetirementDenial::Waiting);
        };
        let Some(displaced) = self.commit_retirement_intent()? else {
            return Ok(None);
        };
        let checkpoint = match covering_checkpoint(checkpoints, displaced.artifact) {
            Ok(completed) => completed,
            Err(denial) => {
                self.revert_retirement_claim(displaced.artifact);
                return Err(denial);
            }
        };
        let captured = checkpoint.basis().source().root().generation();
        let checkpoint_covers_source = match displaced.artifact {
            RetiredArtifact::Arena { .. } => captured >= displaced.source_root,
            _ => captured > displaced.source_root,
        };
        if !checkpoint_covers_source {
            self.revert_retirement_claim(displaced.artifact);
            return Err(PhysicalRetirementDenial::Retained);
        }
        #[cfg(feature = "certification-test-authority")]
        self.pause_retirement_kill(1);
        self.finish_retirement(displaced, &checkpoint)?;
        Ok(Some(displaced))
    }
}

fn covering_checkpoint(
    checkpoints: &PhysicalCheckpointSubmission,
    artifact: RetiredArtifact,
) -> Result<CompletedPhysicalCheckpoint, PhysicalRetirementDenial> {
    // Two retirements sharing a generation must not replay one checkpoint.
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&artifact.generation().to_le_bytes());
    key[8..16].copy_from_slice(&artifact.id().to_le_bytes());
    key[16] = artifact.action_code(false);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .expect("retirement checkpoint deadline is nonzero"),
    );
    let TransitionOutcome::Success(handle) = checkpoints.start(request).into_raw() else {
        return Err(PhysicalRetirementDenial::Checkpoint);
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => Ok(completed),
        PhysicalCheckpointOutcome::ProvenNoEffect(_)
        | PhysicalCheckpointOutcome::Indeterminate(_) => Err(PhysicalRetirementDenial::Checkpoint),
    }
}
