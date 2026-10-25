use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::RelationalNativeCheckpointCaptureDenial;
use worth_relational::facade::publication::PatchStreamPosition;

use super::RelationalStoreHome;
use crate::checkpoint_record::{self, RelationalStoreCheckpointRecordRefusal};

/// A checkpoint the home made the selected recovery basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalStoreCheckpoint {
    covered: PatchStreamPosition,
    covered_transitions: RelationalBranchTransitionSequence,
    generation: u64,
    bytes: usize,
}

impl RelationalStoreCheckpoint {
    /// The last canonical position the checkpoint includes. Recovery replays
    /// only the journal after it.
    pub fn covered(&self) -> PatchStreamPosition {
        self.covered
    }

    /// The last branch transition the checkpoint includes. Recovery replays
    /// only the transitions after it.
    pub fn covered_transitions(&self) -> RelationalBranchTransitionSequence {
        self.covered_transitions
    }

    /// How many checkpoints this home has selected, including this one.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The native checkpoint's size.
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

/// Why a checkpoint did not become the selected recovery basis. The previous
/// checkpoint, if any, remains selected unless the publication is unresolved.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalStoreCheckpointFailure {
    Capture(RelationalNativeCheckpointCaptureDenial),
    /// The captured checkpoint did not report its coverage.
    CoverageUnknown,
    Record(RelationalStoreCheckpointRecordRefusal),
    /// The checkpoint record is selected, but Store's own physical checkpoint
    /// did not complete, so Store recovery still replays the WAL above the
    /// previous physical checkpoint. The home stays correct; recovery is
    /// longer until the next checkpoint succeeds.
    PhysicalCheckpoint {
        detail: String,
    },
}

impl RelationalStoreHome {
    /// Capture Relational's native checkpoint and make it the home's selected
    /// recovery basis in one durable decision.
    pub fn checkpoint(
        &self,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<RelationalStoreCheckpoint, RelationalStoreCheckpointFailure> {
        let checkpoint = self
            .runtime
            .durability_authority()
            .native_checkpoint(policy)
            .map_err(RelationalStoreCheckpointFailure::Capture)?;
        let covered = checkpoint
            .covered_position()
            .ok_or(RelationalStoreCheckpointFailure::CoverageUnknown)?;
        let covered_transitions = checkpoint
            .covered_branch_transitions()
            .ok_or(RelationalStoreCheckpointFailure::CoverageUnknown)?;
        let generation =
            checkpoint_record::publish(&self.serving, covered, covered_transitions, &checkpoint)
                .map_err(RelationalStoreCheckpointFailure::Record)?;
        complete_physical_checkpoint(&self.serving, generation)?;
        Ok(RelationalStoreCheckpoint {
            covered,
            covered_transitions,
            generation,
            bytes: checkpoint.bytes().len(),
        })
    }
}

/// Bound Store's own recovery to the WAL after this checkpoint record. The
/// idempotency key is the record's generation, so a retry of one home
/// checkpoint resolves to the same physical checkpoint.
fn complete_physical_checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    generation: u64,
) -> Result<(), RelationalStoreCheckpointFailure> {
    use worth_proof::TransitionOutcome;
    use worth_store::physical_runtime::{
        PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
        PhysicalCheckpointRequest,
    };
    let failure = |detail: String| RelationalStoreCheckpointFailure::PhysicalCheckpoint { detail };
    let mut key = [0_u8; 32];
    key[..8].copy_from_slice(b"WSRCKPT1");
    key[8..16].copy_from_slice(&generation.to_le_bytes());
    let deadline = PhysicalCheckpointDeadline::after_milliseconds(120_000)
        .ok_or_else(|| failure("checkpoint deadline is not representable".to_owned()))?;
    let request =
        PhysicalCheckpointRequest::fuzzy(PhysicalCheckpointIdempotencyKey::new(key), deadline);
    let handle = match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        other => {
            return Err(failure(format!(
                "physical checkpoint start {}",
                crate::physical_store::unfinished_outcome(&other)
            )));
        }
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => Ok(()),
        other => Err(failure(format!("{other:?}"))),
    }
}
