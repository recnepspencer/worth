use worth_store_physical_backend::{
    ArtifactTreeDirectory, ArtifactTreeFile, QualifiedFilesystemMedia,
};
use worth_store_wal::{
    WalAppendFrontier, WalSegmentArtifactIdentity, WalSegmentGeneration, WalSegmentId,
    WalSegmentScanRecord, WalTopologyScan,
};

use crate::physical_runtime::PhysicalWalPolicy;

use super::{
    PhysicalWalBindingReopenCutoff, PhysicalWalOpenFailure, PhysicalWalSegmentInventory,
    PhysicalWalSegmentInventoryUpdateDenial, ReopenedPhysicalWalInventory,
    ReopenedPhysicalWalMember,
};

mod interrupted_active_tail;
mod trailing_empty_segment;

pub(in crate::physical_runtime) fn reopen_wal_inventory(
    media: &QualifiedFilesystemMedia,
    policy: PhysicalWalPolicy,
    cutoff: PhysicalWalBindingReopenCutoff,
) -> Result<ReopenedPhysicalWalInventory, PhysicalWalOpenFailure> {
    let directory = wal_directory();
    let tree = media.artifact_tree();
    if !tree
        .directory_exists(&directory)
        .map_err(PhysicalWalOpenFailure::Media)?
    {
        tree.create_directory(&directory)
            .map_err(PhysicalWalOpenFailure::Media)?;
    }
    let inventory_limit = usize::try_from(policy.segment_inventory_limit().get().get())
        .map_err(|_| PhysicalWalOpenFailure::InventoryLimitExceeded)?;
    let names = tree
        .list_file_names_bounded(&directory, inventory_limit)
        .map_err(map_listing_failure)?;
    if names.is_empty() {
        return empty_inventory(&directory, cutoff);
    }

    let mut segments = names
        .iter()
        .map(|name| {
            WalSegmentArtifactIdentity::parse(name)
                .ok_or(PhysicalWalOpenFailure::NonCanonicalArtifact)
        })
        .collect::<Result<Vec<_>, _>>()?;
    segments.sort_unstable();
    let trailing_empty = trailing_empty_segment::separate(&tree, &directory, &mut segments)?;

    let byte_limit = policy.segment_byte_limit().get().get();
    let active_identity = *segments
        .last()
        .expect("a nonempty retained inventory has one active segment");
    let mut scans = Vec::with_capacity(segments.len());
    let mut inspections: Vec<worth_store_wal::WalSegmentInspection> =
        Vec::with_capacity(segments.len());
    let mut total_frames = 0u64;
    let mut total_bytes = 0u64;
    let mut peak_buffer_bytes = 0u64;
    let mut active_lsn_end = None;
    let mut members = Vec::new();
    let mut retirement_spans = Vec::new();
    let mut retirement_records = Vec::new();
    let mut retirement_locations = Vec::new();
    let mut interrupted_tail = None;
    let mut interrupted_segment = None;
    for identity in segments.iter().copied() {
        let artifact = artifact(&directory, identity);
        let byte_count = tree
            .file_length(&artifact)
            .map_err(PhysicalWalOpenFailure::Media)?;
        if byte_count == 0 {
            return Err(PhysicalWalOpenFailure::EmptySegment);
        }
        if byte_count > byte_limit {
            return Err(PhysicalWalOpenFailure::SegmentByteLimitExceeded {
                admitted: byte_limit,
                observed: byte_count,
            });
        }
        let allocation = usize::try_from(byte_count)
            .map_err(|_| PhysicalWalOpenFailure::SegmentAllocationRejected)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(allocation)
            .map_err(|_| PhysicalWalOpenFailure::SegmentAllocationRejected)?;
        bytes.resize(allocation, 0);
        tree.read_exact_at(&artifact, 0, &mut bytes)
            .map_err(PhysicalWalOpenFailure::Media)?;
        let admitted = interrupted_active_tail::inspect(
            identity,
            &artifact,
            &bytes,
            identity == active_identity,
        )?;
        let (verified, repair) = match admitted {
            interrupted_active_tail::ActiveTailInspection::Verified {
                prefix,
                interrupted_tail,
            } => (prefix, interrupted_tail),
            interrupted_active_tail::ActiveTailInspection::InterruptedStart(candidate) => {
                let previous = inspections
                    .last()
                    .ok_or(PhysicalWalOpenFailure::SegmentInspection(
                        worth_store_wal::WalArtifactStoreDenial::InvalidFrame,
                    ))?
                    .identity();
                interrupted_segment = Some(candidate.admit_after(previous)?);
                break;
            }
        };
        if let Some(repair) = repair {
            interrupted_tail = Some(repair);
        }
        for frame in verified.frames().iter().copied() {
            if super::super::super::retention::payload_is_retirement(frame.payload()) {
                let Some(record) = super::super::super::retention::decode_retirement(frame.payload())
                else {
                    return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
                };
                retirement_records.push(record);
                retirement_locations.push((
                    frame.lsn_range().start().get(),
                    frame.lsn_range().end_exclusive().get(),
                ));
                let range = frame.lsn_range();
                match cutoff.lsn() {
                    Some(cutoff_lsn) if range.end_exclusive() <= cutoff_lsn => {}
                    Some(cutoff_lsn) if range.start() < cutoff_lsn => {
                        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
                    }
                    _ => retirement_spans.push((range.start().get(), range.end_exclusive().get())),
                }
                continue;
            }
            if let Some(member) = ReopenedPhysicalWalMember::decode_retained_frame(cutoff, frame)
                .map_err(|_denial| PhysicalWalOpenFailure::MemberPayloadRejected)?
            {
                members.push(member);
            }
        }
        let inspection = verified.inspection();
        total_frames = total_frames
            .checked_add(inspection.frame_count())
            .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
        total_bytes = total_bytes
            .checked_add(inspection.byte_count())
            .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
        peak_buffer_bytes = peak_buffer_bytes.max(inspection.byte_count());
        active_lsn_end = Some(inspection.lsn_range().end_exclusive());
        inspections.push(inspection);
        scans.push(WalSegmentScanRecord::current(
            identity.segment(),
            identity.generation(),
            inspection.lsn_range(),
        ));
    }
    let generation = inspections
        .first()
        .expect("an admitted WAL inventory retains a verified segment")
        .identity()
        .generation();
    WalTopologyScan::from_segment_scan(scans)
        .admit_replay_cursor(generation)
        .map_err(|denial| PhysicalWalOpenFailure::Topology(denial.kind()))?;
    let active = inspections
        .last()
        .expect("a nonempty inspected WAL inventory has one active segment")
        .identity();
    let segment_count = inspections.len() as u32;
    let active_artifact = artifact(&directory, active);
    let active_bytes = inspections
        .last()
        .expect("a nonempty inspected WAL inventory has one active segment")
        .byte_count();
    let active_lsn_end =
        active_lsn_end.expect("a nonempty inspected WAL inventory has one active LSN frontier");
    let segment_inventory =
        PhysicalWalSegmentInventory::from_reopened(inspections).map_err(map_inventory_failure)?;
    require_checkpoint_cutoff_within_retained_wal(cutoff, &segment_inventory, active_lsn_end)?;
    if let Some(interrupted_tail) = interrupted_tail {
        interrupted_tail.truncate_durably(&tree)?;
    }
    if let Some(interrupted_segment) = interrupted_segment {
        interrupted_segment.remove_durably(&tree)?;
    }
    if let Some(trailing_empty) = trailing_empty {
        trailing_empty.remove_durably(&tree)?;
    }
    let requires_inspection =
        cutoff.lsn().is_none() && !segment_inventory.retains_canonical_wal_origin();
    Ok(ReopenedPhysicalWalInventory {
        frontier: WalAppendFrontier::observed(
            active.segment(),
            active.generation(),
            active_bytes,
            active_lsn_end,
        ),
        active_artifact,
        segment_count,
        frame_count: total_frames,
        byte_count: total_bytes,
        peak_buffer_bytes,
        requires_inspection,
        segments: segment_inventory,
        members,
        retirement_spans,
        retirement_records,
        retirement_locations,
    })
}

fn require_checkpoint_cutoff_within_retained_wal(
    cutoff: PhysicalWalBindingReopenCutoff,
    inventory: &PhysicalWalSegmentInventory,
    active_lsn_end: worth_store_wal::LogSequenceNumber,
) -> Result<(), PhysicalWalOpenFailure> {
    let Some(cutoff) = cutoff.lsn() else {
        return Ok(());
    };
    let first = inventory
        .first_lsn_start()
        .ok_or(PhysicalWalOpenFailure::CheckpointCutoffOutsideRetainedWal)?;
    if cutoff < first || cutoff > active_lsn_end {
        return Err(PhysicalWalOpenFailure::CheckpointCutoffOutsideRetainedWal);
    }
    Ok(())
}

fn map_inventory_failure(
    denial: PhysicalWalSegmentInventoryUpdateDenial,
) -> PhysicalWalOpenFailure {
    let kind = match denial {
        PhysicalWalSegmentInventoryUpdateDenial::ArtifactOrder => {
            worth_store_wal::WalTopologyDenialKind::StaleSegment
        }
        PhysicalWalSegmentInventoryUpdateDenial::GenerationMismatch => {
            worth_store_wal::WalTopologyDenialKind::WrongGeneration
        }
        PhysicalWalSegmentInventoryUpdateDenial::LsnDiscontinuity => {
            worth_store_wal::WalTopologyDenialKind::Gap
        }
        PhysicalWalSegmentInventoryUpdateDenial::ByteCountOverflow => {
            return PhysicalWalOpenFailure::CounterOverflow;
        }
    };
    PhysicalWalOpenFailure::Topology(kind)
}

fn empty_inventory(
    directory: &ArtifactTreeDirectory,
    cutoff: PhysicalWalBindingReopenCutoff,
) -> Result<ReopenedPhysicalWalInventory, PhysicalWalOpenFailure> {
    if cutoff.lsn().is_some() {
        return Err(PhysicalWalOpenFailure::CheckpointCutoffOutsideRetainedWal);
    }
    let segment = WalSegmentId::new(1).expect("the initial WAL segment is nonzero");
    let generation = WalSegmentGeneration::new(1).expect("the initial WAL generation is nonzero");
    Ok(ReopenedPhysicalWalInventory {
        frontier: WalAppendFrontier::empty(segment, generation),
        active_artifact: artifact(
            directory,
            WalSegmentArtifactIdentity::new(segment, generation),
        ),
        segment_count: 0,
        frame_count: 0,
        byte_count: 0,
        peak_buffer_bytes: 0,
        requires_inspection: false,
        segments: PhysicalWalSegmentInventory::empty(),
        members: Vec::new(),
        retirement_spans: Vec::new(),
        retirement_records: Vec::new(),
        retirement_locations: Vec::new(),
    })
}

fn wal_directory() -> ArtifactTreeDirectory {
    ArtifactTreeDirectory::families()
        .child("wal")
        .expect("the Store-owned WAL directory is portable")
}

fn artifact(
    directory: &ArtifactTreeDirectory,
    identity: WalSegmentArtifactIdentity,
) -> ArtifactTreeFile {
    directory
        .file(&identity.file_name())
        .expect("canonical WAL artifact names are portable")
}

fn map_listing_failure(
    failure: worth_store_physical_backend::ArtifactTreeFailure,
) -> PhysicalWalOpenFailure {
    if matches!(
        failure.kind(),
        worth_store_physical_backend::ArtifactTreeFailureKind::AccessLimitExceeded
    ) {
        PhysicalWalOpenFailure::InventoryLimitExceeded
    } else {
        PhysicalWalOpenFailure::Media(failure)
    }
}
