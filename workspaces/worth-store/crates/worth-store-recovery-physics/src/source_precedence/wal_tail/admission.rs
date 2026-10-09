//! Allocation-free partitioning and validation into caller-prepared tail storage.

use super::{
    continuation, PhysicalWalSegmentCandidate, SelectedPhysicalWalTail,
    SelectedPhysicalWalTailDenial,
};
use crate::source_precedence::CheckpointCoveredWalArtifact;

pub fn admit_physical_wal_tail(
    checkpoint_frontier: u64,
    checkpoint_cutoff: Option<u64>,
    mut candidates: Vec<PhysicalWalSegmentCandidate>,
    mut checkpoint_covered: Vec<CheckpointCoveredWalArtifact>,
    mut retained: Vec<PhysicalWalSegmentCandidate>,
) -> Result<SelectedPhysicalWalTail, SelectedPhysicalWalTailDenial> {
    require_prepared_storage(candidates.len(), &checkpoint_covered, &retained)?;
    candidates.sort_unstable_by_key(|candidate| candidate.identity());
    if candidates
        .windows(2)
        .any(|pair| pair[0].identity() == pair[1].identity())
    {
        return Err(SelectedPhysicalWalTailDenial::DuplicateArtifact);
    }
    for candidate in candidates {
        if candidate.inspection().lsn_range().end_exclusive().get() <= checkpoint_frontier {
            checkpoint_covered.push(CheckpointCoveredWalArtifact::from_candidate(candidate));
            continue;
        }
        if let Some(candidate) = candidate.trim_before(checkpoint_frontier)? {
            retained.push(candidate);
        }
    }
    candidates = retained;
    let mut frame_count = 0_u64;
    let mut byte_count = 0_u64;
    for (index, candidate) in candidates.iter().enumerate() {
        crate::wal_prefix::classify_terminal_interruption(
            index,
            candidates.len(),
            candidate.interrupted_tail(),
        )
        .map_err(map_prefix_denial)?;
        if let Some(previous) = index.checked_sub(1).map(|prior| &candidates[prior]) {
            if previous.identity() == candidate.identity() {
                return Err(SelectedPhysicalWalTailDenial::DuplicateArtifact);
            }
            if previous.identity().generation() != candidate.identity().generation() {
                return Err(SelectedPhysicalWalTailDenial::GenerationMismatch);
            }
            if previous.identity().segment().get().checked_add(1)
                != Some(candidate.identity().segment().get())
            {
                return Err(SelectedPhysicalWalTailDenial::SegmentGap);
            }
        }
        frame_count = frame_count
            .checked_add(candidate.selected_frame_count())
            .ok_or(SelectedPhysicalWalTailDenial::CounterOverflow)?;
        byte_count = byte_count
            .checked_add(candidate.selected_byte_count())
            .ok_or(SelectedPhysicalWalTailDenial::CounterOverflow)?;
    }
    crate::wal_prefix::require_contiguous_prefix(
        checkpoint_frontier,
        candidates
            .iter()
            .map(PhysicalWalSegmentCandidate::selected_range),
    )
    .map_err(map_prefix_denial)?;
    let protected_covered_start = continuation::protected_covered_start(
        checkpoint_frontier,
        checkpoint_cutoff,
        &mut checkpoint_covered,
        &candidates,
    )?;
    let facts = crate::wal_prefix::WalValidPrefixFacts {
        frame_count,
        byte_count,
    };
    Ok(SelectedPhysicalWalTail {
        segments: candidates,
        checkpoint_covered,
        protected_covered_start,
        admitted_frontier: checkpoint_frontier,
        admitted_cutoff: checkpoint_cutoff,
        frame_count: facts.frame_count,
        byte_count: facts.byte_count,
    })
}

fn map_prefix_denial(
    denial: crate::wal_prefix::WalPrefixAdmissionDenial,
) -> SelectedPhysicalWalTailDenial {
    match denial {
        crate::wal_prefix::WalPrefixAdmissionDenial::FrontierMismatch => {
            SelectedPhysicalWalTailDenial::CheckpointFrontierMismatch
        }
        crate::wal_prefix::WalPrefixAdmissionDenial::Gap => SelectedPhysicalWalTailDenial::LsnGap,
        crate::wal_prefix::WalPrefixAdmissionDenial::InterruptedMiddle => {
            SelectedPhysicalWalTailDenial::InterruptedMiddleSegment
        }
    }
}

fn require_prepared_storage(
    required: usize,
    covered: &Vec<CheckpointCoveredWalArtifact>,
    retained: &Vec<PhysicalWalSegmentCandidate>,
) -> Result<(), SelectedPhysicalWalTailDenial> {
    if !covered.is_empty()
        || !retained.is_empty()
        || covered.capacity() < required
        || retained.capacity() < required
    {
        return Err(SelectedPhysicalWalTailDenial::PreparedStorage {
            required,
            covered_capacity: covered.capacity(),
            retained_capacity: retained.capacity(),
            covered_length: covered.len(),
            retained_length: retained.len(),
        });
    }
    Ok(())
}
