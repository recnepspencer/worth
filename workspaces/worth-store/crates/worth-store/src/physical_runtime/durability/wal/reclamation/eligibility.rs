use worth_proof::NonEmpty;

use super::super::runtime_owner::PhysicalWalRuntimeState;
use super::EligiblePhysicalWalReclamation;
use crate::physical_runtime::durability::checkpoint::NamespaceDurableCheckpointPublication;

pub(in crate::physical_runtime::durability) enum PhysicalWalReclamationPlan {
    NotRequired {
        checkpoint: worth_store_physical_format::PhysicalCheckpointIdentity,
    },
    Required(EligiblePhysicalWalReclamation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::durability) enum PhysicalWalReclamationEligibilityDenial {
    CheckpointIdentityMismatch,
    CutoffBeforeCheckpointBoundary,
    RetainedTailMismatch,
    LiveInventoryMismatch,
    CandidateCrossesCheckpointBoundary,
}

pub(in crate::physical_runtime::durability) fn plan_reclamation(
    state: &PhysicalWalRuntimeState,
    publication: &NamespaceDurableCheckpointPublication,
) -> Result<PhysicalWalReclamationPlan, PhysicalWalReclamationEligibilityDenial> {
    let checkpoint = publication.basis().identity();
    let tail = publication.retained_wal_tail();
    let compaction = publication.binding_compaction();
    if tail.checkpoint_identity() != checkpoint || compaction.checkpoint_identity() != checkpoint {
        return Err(PhysicalWalReclamationEligibilityDenial::CheckpointIdentityMismatch);
    }
    let checkpoint_boundary = tail.checkpoint_boundary_lsn();
    if compaction.wal_cutoff_lsn_exclusive() < checkpoint_boundary.get() {
        return Err(PhysicalWalReclamationEligibilityDenial::CutoffBeforeCheckpointBoundary);
    }
    let first_retained = tail
        .segments()
        .first()
        .expect("a retained WAL tail is nonempty");
    let entries = state.segments.entries();
    let allow_extension = tail.segments().len() == 1;
    let retained_index = entries
        .iter()
        .position(|entry| covers_retained(entry, first_retained, allow_extension))
        .ok_or(PhysicalWalReclamationEligibilityDenial::RetainedTailMismatch)?;
    require_retained_suffix(entries, retained_index, tail.segments())?;
    if retained_index == 0 {
        return Ok(PhysicalWalReclamationPlan::NotRequired { checkpoint });
    }
    let candidates = reclaimable_before_retirement(
        &entries[..retained_index],
        &state.unresolved_retirement_spans,
    );
    if candidates.is_empty() {
        return Ok(PhysicalWalReclamationPlan::NotRequired { checkpoint });
    }
    if candidates
        .iter()
        .any(|entry| entry.lsn_range().end_exclusive() > checkpoint_boundary)
    {
        return Err(PhysicalWalReclamationEligibilityDenial::CandidateCrossesCheckpointBoundary);
    }
    let candidates = NonEmpty::try_from_vec(candidates.to_vec())
        .expect("a nonzero retained index has a nonempty reclaimable prefix");
    Ok(PhysicalWalReclamationPlan::Required(
        EligiblePhysicalWalReclamation::new(
            checkpoint,
            compaction.generation().get(),
            compaction.records_digest(),
            checkpoint_boundary,
            candidates,
        ),
    ))
}

fn covers_retained(
    entry: &super::super::inventory::PhysicalWalSegmentInventoryEntry,
    retained: &crate::physical_runtime::RetainedWalSegment,
    allow_extension: bool,
) -> bool {
    if entry.identity() != retained.artifact()
        || entry.lsn_range().start() != retained.observed_lsn_range().start()
    {
        return false;
    }
    if allow_extension {
        entry.lsn_range().end_exclusive() >= retained.observed_lsn_range().end_exclusive()
            && entry.byte_count() >= retained.physical_bytes()
    } else {
        entry.lsn_range() == retained.observed_lsn_range()
            && entry.byte_count() == retained.physical_bytes()
    }
}

fn reclaimable_before_retirement<'a>(
    candidates: &'a [super::super::inventory::PhysicalWalSegmentInventoryEntry],
    holds: &[(
        crate::physical_runtime::durability::RetiredArtifact,
        u64,
        u64,
    )],
) -> &'a [super::super::inventory::PhysicalWalSegmentInventoryEntry] {
    let Some(index) = candidates.iter().position(|entry| {
        let start = entry.lsn_range().start().get();
        let end = entry.lsn_range().end_exclusive().get();
        holds
            .iter()
            .any(|(_, hold_start, hold_end)| start < *hold_end && *hold_start < end)
    }) else {
        return candidates;
    };
    &candidates[..index]
}

fn require_retained_suffix(
    inventory: &[super::super::inventory::PhysicalWalSegmentInventoryEntry],
    start: usize,
    retained: &[crate::physical_runtime::RetainedWalSegment],
) -> Result<(), PhysicalWalReclamationEligibilityDenial> {
    let end = start
        .checked_add(retained.len())
        .ok_or(PhysicalWalReclamationEligibilityDenial::LiveInventoryMismatch)?;
    let available = inventory
        .get(start..end)
        .ok_or(PhysicalWalReclamationEligibilityDenial::LiveInventoryMismatch)?;
    let matches =
        available
            .iter()
            .zip(retained)
            .enumerate()
            .all(|(index, (entry, retained_segment))| {
                covers_retained(entry, retained_segment, index + 1 == retained.len())
            });
    matches
        .then_some(())
        .ok_or(PhysicalWalReclamationEligibilityDenial::LiveInventoryMismatch)
}
