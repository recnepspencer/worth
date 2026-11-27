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
    let candidates = reclaimable_closed_prefix(
        &entries[..retained_index],
        &state.unresolved_retirement_spans,
        &state.copy_obligations,
        compaction.wal_cutoff_lsn_exclusive(),
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

/// Whether a single closed WAL segment is eligible for reclamation at
/// `cutoff_lsn_exclusive`, i.e., free of overlapping retirement holds and
/// unresolved copy obligations.
pub(in crate::physical_runtime::durability) fn segment_is_reclaimable(
    entry: super::super::inventory::PhysicalWalSegmentInventoryEntry,
    retirement_holds: &[(
        crate::physical_runtime::durability::RetiredArtifact,
        u64,
        u64,
    )],
    copy_obligations: &[super::super::RetainedExtentCopyObligation],
    cutoff_lsn_exclusive: u64,
) -> bool {
    let start = entry.lsn_range().start().get();
    let end = entry.lsn_range().end_exclusive().get();
    let held_by_retirement = retirement_holds
        .iter()
        .any(|(_, hold_start, hold_end)| start < *hold_end && *hold_start < end);
    if held_by_retirement {
        return false;
    }
    !copy_obligations
        .iter()
        .any(|copy| copy.holds_at(cutoff_lsn_exclusive) && copy.overlaps(start, end))
}

/// Prefix of `closed` segments eligible for reclamation at `cutoff_lsn_exclusive`,
/// stopping at the first segment held by an unresolved retirement or copy obligation.
pub(in crate::physical_runtime::durability) fn reclaimable_closed_prefix<'a>(
    closed: &'a [super::super::inventory::PhysicalWalSegmentInventoryEntry],
    retirement_holds: &[(
        crate::physical_runtime::durability::RetiredArtifact,
        u64,
        u64,
    )],
    copy_obligations: &[super::super::RetainedExtentCopyObligation],
    cutoff_lsn_exclusive: u64,
) -> &'a [super::super::inventory::PhysicalWalSegmentInventoryEntry] {
    match closed.iter().position(|entry| {
        !segment_is_reclaimable(
            *entry,
            retirement_holds,
            copy_obligations,
            cutoff_lsn_exclusive,
        )
    }) {
        Some(index) => &closed[..index],
        None => closed,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_runtime::durability::wal::copy_obligation::observe_copy_record;
    use crate::physical_runtime::durability::wal::inventory::PhysicalWalSegmentInventoryEntry;
    use worth_store_physical_format::{
        DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange, PersistedRecordIdentity,
        PhysicalExtentCopyIntent, PhysicalExtentCopyRecord, PhysicalExtentCopyResolution,
        PhysicalExtentCopyResolutionKind, PhysicalExtentId, PhysicalGeneration,
        PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration,
    };
    use worth_store_wal::{
        LogSequenceNumber, WalLsnRange, WalSegmentArtifactIdentity, WalSegmentGeneration,
        WalSegmentId,
    };

    fn entry(segment: u64, start: u64, end: u64) -> PhysicalWalSegmentInventoryEntry {
        PhysicalWalSegmentInventoryEntry::from_parts_for_test(
            WalSegmentArtifactIdentity::new(
                WalSegmentId::new(segment).unwrap(),
                WalSegmentGeneration::new(1).unwrap(),
            ),
            WalLsnRange::new(LogSequenceNumber::new(start), LogSequenceNumber::new(end)).unwrap(),
            128,
        )
    }

    #[test]
    fn unresolved_copy_obligation_blocks_reclaimable_closed_prefix() {
        let closed = [entry(1, 1, 10), entry(2, 10, 20)];
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let source = DurableExtentRecordPlacement::legacy_unknown(
            PersistedRecordIdentity::new([7; 16], 9).unwrap(),
            PhysicalGenerationAuthority::for_canonical_physical_format()
                .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
                .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap()),
            40_000,
            ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 53_248, 53_248).unwrap(),
        )
        .unwrap();
        let intent = PhysicalExtentCopyIntent::new(
            format,
            [19; 32],
            12,
            source,
            ExtentArenaRange::new(ExtentArenaId::new(8).unwrap(), 0, 53_248).unwrap(),
            4096,
            [23; 32],
        )
        .unwrap();
        let mut obligations = Vec::new();
        observe_copy_record(
            &mut obligations,
            PhysicalExtentCopyRecord::Intent(intent),
            1,
            1,
            3,
            4,
            0,
        )
        .unwrap();
        // While copy intent in segment 1 is unresolved, closed prefix is empty.
        assert!(reclaimable_closed_prefix(&closed, &[], &obligations, 25).is_empty());
        // Once resolved at LSN 15..16, a checkpoint at cutoff 25 can reclaim segment 1 & 2.
        let resolution = PhysicalExtentCopyResolution::new(
            intent.operation(),
            obligations[0].intent_digest(),
            3,
            PhysicalExtentCopyResolutionKind::Cancelled,
        )
        .unwrap();
        observe_copy_record(
            &mut obligations,
            PhysicalExtentCopyRecord::Resolved(resolution),
            2,
            1,
            15,
            16,
            0,
        )
        .unwrap();
        assert_eq!(
            reclaimable_closed_prefix(&closed, &[], &obligations, 25).len(),
            2
        );
    }
}
