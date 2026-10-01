use worth_store_wal::WalSegmentArtifactIdentity;

use super::{PhysicalWalSegmentCandidate, SelectedPhysicalWalTailDenial};
use crate::source_precedence::CheckpointCoveredWalArtifact;

/// The selected checkpoint's LSN cutoff needs an original physical WAL
/// identity after recovery. Keep exactly the verified covered suffix needed
/// to connect that cutoff to the first replay artifact (or to the checkpoint
/// frontier when there is no replay artifact).
pub(super) fn protected_covered_start(
    frontier: u64,
    cutoff: Option<u64>,
    covered: &mut Vec<CheckpointCoveredWalArtifact>,
    replay: &[PhysicalWalSegmentCandidate],
) -> Result<usize, SelectedPhysicalWalTailDenial> {
    let Some(cutoff) = cutoff else {
        return Ok(covered.len());
    };
    if cutoff > frontier {
        return Err(SelectedPhysicalWalTailDenial::CheckpointFrontierMismatch);
    }
    let first_replay = replay.first().map(PhysicalWalSegmentCandidate::inspection);
    if first_replay.is_some_and(|first| {
        let range = first.lsn_range();
        range.start().get() <= cutoff && cutoff < range.end_exclusive().get()
    }) {
        return Ok(covered.len());
    }

    // This Vec is already bounded by C9 WAL inventory admission. Sorting it
    // in place avoids a second, uncharged recovery-memory allocation.
    covered.sort_unstable_by_key(|artifact| {
        (
            artifact.lsn_range().end_exclusive().get(),
            artifact.identity(),
        )
    });
    let mut required_end = first_replay.map_or(frontier, |first| first.lsn_range().start().get());
    let mut next_identity = first_replay.map(|first| first.identity());
    let mut protected_count = 0_usize;
    loop {
        let index = covered
            .partition_point(|artifact| artifact.lsn_range().end_exclusive().get() < required_end);
        let artifact = covered
            .get(index)
            .filter(|artifact| artifact.lsn_range().end_exclusive().get() == required_end)
            .ok_or(SelectedPhysicalWalTailDenial::CheckpointContinuationMissing)?;
        if covered
            .get(index + 1)
            .is_some_and(|next| next.lsn_range().end_exclusive().get() == required_end)
        {
            return Err(SelectedPhysicalWalTailDenial::CheckpointContinuationAmbiguous);
        }
        if !artifact.cleanup_safe() {
            return Err(SelectedPhysicalWalTailDenial::CheckpointContinuationInterrupted);
        }
        if let Some(next) = next_identity {
            require_successor(artifact.identity(), next)?;
        }
        let start = artifact.lsn_range().start().get();
        let identity = artifact.identity();
        protected_count += 1;
        if start <= cutoff {
            covered.sort_unstable_by_key(CheckpointCoveredWalArtifact::identity);
            let first = covered.partition_point(|artifact| artifact.identity() < identity);
            if protected_count != covered.len() - first {
                return Err(SelectedPhysicalWalTailDenial::CheckpointContinuationDiscontinuous);
            }
            return Ok(first);
        }
        next_identity = Some(identity);
        required_end = start;
    }
}

fn require_successor(
    previous: WalSegmentArtifactIdentity,
    next: WalSegmentArtifactIdentity,
) -> Result<(), SelectedPhysicalWalTailDenial> {
    if previous.generation() != next.generation()
        || previous.segment().get().checked_add(1) != Some(next.segment().get())
    {
        return Err(SelectedPhysicalWalTailDenial::CheckpointContinuationDiscontinuous);
    }
    Ok(())
}
