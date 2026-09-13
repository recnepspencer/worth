use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, RecordArtifactFile,
};

use super::{frontier, inventory, CandidateBuildDenial};
use crate::entry::{
    PhysicalRecoverySuccessorCandidateDenial, PhysicalRecoverySuccessorCandidateMismatch,
};
use crate::progression::planned::basis::{
    RecoveryBaseImagePlan, RecoveryObservedSuccessorCandidate, RecoverySelectedSourceInventory,
};

pub(super) fn admit_observed(
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    observed: &RecoveryObservedSuccessorCandidate,
) -> Result<(), CandidateBuildDenial> {
    let root = &observed.root;
    let selected = base.selected_root();
    let free = &observed.free_space;
    let last_inline_record = final_inventory
        .last_inline_record
        .or(selected.last_inline_record());
    let last_inline_segment = final_inventory
        .last_inline_segment
        .or(selected.last_inline_segment());
    admit_equal(
        root.generation(),
        base.destination_generation(),
        root.generation(),
        PhysicalRecoverySuccessorCandidateMismatch::RootGeneration {
            expected: base.destination_generation(),
            observed: root.generation(),
        },
    )?;
    admit_equal(
        root.generation(),
        selected.tree_identity(),
        root.tree_identity(),
        PhysicalRecoverySuccessorCandidateMismatch::RootTreeIdentity {
            expected: selected.tree_identity(),
            observed: root.tree_identity(),
        },
    )?;
    admit_equal(
        root.generation(),
        final_inventory.capacity as u64,
        root.node_capacity() as u64,
        PhysicalRecoverySuccessorCandidateMismatch::RootNodeCapacity {
            expected: final_inventory.capacity,
            observed: root.node_capacity(),
        },
    )?;
    admit_equal(
        root.generation(),
        final_inventory.placements.len() as u64,
        root.record_count(),
        PhysicalRecoverySuccessorCandidateMismatch::RootRecordCount {
            expected: final_inventory.placements.len() as u64,
            observed: root.record_count(),
        },
    )?;
    admit_frontiers(observed, selected, source, root, free)?;
    admit(
        root.generation(),
        root.last_inline_record() == last_inline_record,
        PhysicalRecoverySuccessorCandidateMismatch::RootLastInlineRecord,
    )?;
    admit(
        root.generation(),
        root.last_inline_segment() == last_inline_segment,
        PhysicalRecoverySuccessorCandidateMismatch::RootLastInlineSegment,
    )?;
    admit(
        root.generation(),
        observed.placements.as_ref() == final_inventory.placements,
        PhysicalRecoverySuccessorCandidateMismatch::RecordPlacements,
    )?;
    admit(
        root.generation(),
        observed.segment_entries.as_ref() == final_inventory.segments,
        PhysicalRecoverySuccessorCandidateMismatch::SegmentMembership,
    )?;
    admit(
        root.generation(),
        observed.free_entries.as_ref() == final_inventory.free,
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceMembership,
    )?;
    admit_equal(
        root.generation(),
        root.generation(),
        free.generation(),
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceGeneration {
            expected: root.generation(),
            observed: free.generation(),
        },
    )?;
    admit_equal(
        root.generation(),
        source.free_space.tree_identity(),
        free.tree_identity(),
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceTreeIdentity {
            expected: source.free_space.tree_identity(),
            observed: free.tree_identity(),
        },
    )?;
    admit_equal(
        root.generation(),
        final_inventory.capacity as u64,
        free.node_capacity() as u64,
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceNodeCapacity {
            expected: final_inventory.capacity,
            observed: free.node_capacity(),
        },
    )?;
    admit_equal(
        root.generation(),
        source.free_space.segment_page_capacity() as u64,
        free.segment_page_capacity() as u64,
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceSegmentPageCapacity {
            expected: source.free_space.segment_page_capacity(),
            observed: free.segment_page_capacity(),
        },
    )?;
    admit_equal(
        root.generation(),
        final_inventory.free.len() as u64,
        free.entry_count(),
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceEntryCount {
            expected: final_inventory.free.len() as u64,
            observed: free.entry_count(),
        },
    )?;
    admit_equal(
        root.generation(),
        final_inventory.next_segment,
        free.next_segment(),
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceNextSegment {
            expected: final_inventory.next_segment,
            observed: free.next_segment(),
        },
    )?;
    admit_equal(
        root.generation(),
        final_inventory.next_page,
        free.next_page(),
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceNextPage {
            expected: final_inventory.next_page,
            observed: free.next_page(),
        },
    )?;
    admit_equal(
        root.generation(),
        final_inventory.next_extent,
        free.next_extent(),
        PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceNextExtent {
            expected: final_inventory.next_extent,
            observed: free.next_extent(),
        },
    )?;
    Ok(())
}

fn admit_equal(
    generation: u64,
    expected: u64,
    observed: u64,
    mismatch: PhysicalRecoverySuccessorCandidateMismatch,
) -> Result<(), CandidateBuildDenial> {
    admit(generation, expected == observed, mismatch)
}

fn admit(
    generation: u64,
    matches: bool,
    mismatch: PhysicalRecoverySuccessorCandidateMismatch,
) -> Result<(), CandidateBuildDenial> {
    matches
        .then_some(())
        .ok_or_else(|| conflict(generation, mismatch))
}

fn conflict(
    generation: u64,
    mismatch: PhysicalRecoverySuccessorCandidateMismatch,
) -> CandidateBuildDenial {
    conflict_at(
        generation,
        RecordArtifactFile::RootManifest { generation },
        mismatch,
    )
}

fn conflict_at(
    generation: u64,
    artifact: RecordArtifactFile,
    mismatch: PhysicalRecoverySuccessorCandidateMismatch,
) -> CandidateBuildDenial {
    CandidateBuildDenial::SuccessorCandidate(PhysicalRecoverySuccessorCandidateDenial::Conflict {
        artifact,
        generation,
        mismatch,
    })
}

fn admit_frontiers(
    observed: &RecoveryObservedSuccessorCandidate,
    selected: &DurablePhysicalRootManifest,
    source: &RecoverySelectedSourceInventory,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
) -> Result<(), CandidateBuildDenial> {
    let generation = root.generation();
    let frontiers = exact_successor_frontiers(
        observed,
        generation,
        (selected.next_block(), root.next_block()),
        (selected.next_segment_block(), root.next_segment_block()),
        (source.free_space.next_block(), free.next_block()),
    );
    match frontiers {
        Ok(()) => Ok(()),
        Err(mismatch) => Err(conflict(generation, mismatch)),
    }
}

fn exact_successor_frontiers(
    observed: &RecoveryObservedSuccessorCandidate,
    generation: u64,
    root: (u64, u64),
    segment: (u64, u64),
    free: (u64, u64),
) -> Result<(), PhysicalRecoverySuccessorCandidateMismatch> {
    let mut root_blocks = Vec::new();
    let mut segment_blocks = Vec::new();
    let mut free_blocks = Vec::new();
    for candidate in &observed.artifacts {
        match candidate.artifact {
            RecordArtifactFile::RootRoutingBlock {
                generation: found,
                block,
            } if found == generation => root_blocks.push(block),
            RecordArtifactFile::SegmentMembershipBlock {
                generation: found,
                block,
            } if found == generation => segment_blocks.push(block),
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation: found,
                block,
            } if found == generation => free_blocks.push(block),
            RecordArtifactFile::RootRoutingBlock { .. }
            | RecordArtifactFile::SegmentMembershipBlock { .. }
            | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => {
                return Err(PhysicalRecoverySuccessorCandidateMismatch::RootRoutingFrontier)
            }
            _ => {}
        }
    }
    if !frontier::exact_contiguous_blocks(&mut root_blocks, root.0, root.1) {
        return Err(PhysicalRecoverySuccessorCandidateMismatch::RootRoutingFrontier);
    }
    if !frontier::exact_contiguous_blocks(&mut segment_blocks, segment.0, segment.1) {
        return Err(PhysicalRecoverySuccessorCandidateMismatch::SegmentMembershipFrontier);
    }
    if !frontier::exact_contiguous_blocks(&mut free_blocks, free.0, free.1) {
        return Err(PhysicalRecoverySuccessorCandidateMismatch::FreeSpaceMembershipFrontier);
    }
    Ok(())
}
