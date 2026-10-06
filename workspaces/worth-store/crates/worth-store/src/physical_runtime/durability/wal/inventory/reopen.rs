use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_wal::{
    WalAppendFrontier, WalSegmentArtifactIdentity, WalSegmentScanRecord, WalTopologyScan,
};

use crate::physical_runtime::PhysicalWalPolicy;

use super::{
    PhysicalWalBindingReopenCutoff, PhysicalWalOpenFailure, PhysicalWalSegmentInventory,
    PhysicalWalSegmentInventoryUpdateDenial, ReopenedPhysicalWalInventory,
    ReopenedPhysicalWalMember,
};

mod checkpoint_cutoff;
mod empty;
mod interrupted_active_tail;
mod manifest_residue;
mod paths;
mod tier_epoch;
mod trailing_empty_segment;
use checkpoint_cutoff::require_checkpoint_cutoff_within_retained_wal;
use empty::empty_inventory;
use paths::{artifact, map_listing_failure, wal_directory};

pub(in crate::physical_runtime) fn reopen_wal_inventory(
    media: &QualifiedFilesystemMedia,
    policy: PhysicalWalPolicy,
    cutoff: PhysicalWalBindingReopenCutoff,
    record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    binding_context: crate::physical_runtime::durability::PhysicalBindingDecodingContext,
    reopen_grant: &worth_store_buffer_pool::OperationAllocationGrant,
) -> Result<ReopenedPhysicalWalInventory, PhysicalWalOpenFailure> {
    if reopen_grant.scope() != worth_store_buffer_pool::PhysicalOperationAllocationScope::Recovery {
        return Err(PhysicalWalOpenFailure::ReopenAllocationRejected);
    }
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
        return empty_inventory(&directory, cutoff, record_format);
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
    let mut publication_roster =
        super::publication_roster::PublicationRoster::new(reopen_grant.bytes());
    let mut peak_buffer_bytes = 0u64;
    let mut active_lsn_end = None;
    let mut members = Vec::new();
    let mut retirement_spans = Vec::new();
    let mut copy_obligations = Vec::new();
    let mut retirement_records = Vec::new();
    let mut retained_maintenance = Vec::new();
    let mut release_metadata = std::collections::BTreeMap::new();
    let mut retirement_locations = Vec::new();
    let mut interrupted_tail = None;
    let mut interrupted_segment = None;
    let mut semantic_failure = None;
    let mut retained_released_drop = false;
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
        // The exact on-disk length, not the policy maximum, is the live
        // segment buffer reserve during this sequential inspection.
        publication_roster.admitted_frame_view_count(byte_count)?;
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
            publication_roster.admitted_frame_view_count(byte_count)?,
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
        let active_frame_views_bytes =
            super::publication_roster::PublicationRoster::frame_view_capacity_ceiling(
                verified.frames().len(),
            )?;
        let mut frame_offset = 0_u64;
        for frame in verified.frames().iter().copied() {
            let offset = frame_offset;
            frame_offset = frame_offset
                .checked_add(frame.encoded_bytes())
                .ok_or(PhysicalWalOpenFailure::CounterOverflow)?;
            if semantic_failure.is_some() {
                continue;
            }
            let semantic_observation = (|| -> Result<(), PhysicalWalOpenFailure> {
                if worth_store_physical_format::payload_is_extent_copy_any(frame.payload()) {
                    let record = worth_store_physical_format::PhysicalExtentCopyRecord::decode(
                        frame.payload(),
                        record_format,
                    )
                    .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
                    let range = frame.lsn_range();
                    let start = range.start().get();
                    let end = range.end_exclusive().get();
                    let checkpoint = cutoff.lsn().map_or(0, |lsn| lsn.get());
                    super::super::copy_obligation::observe_copy_record(
                        &mut copy_obligations,
                        record,
                        identity.segment().get(),
                        identity.generation().get(),
                        start,
                        end,
                        checkpoint,
                    )
                    .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
                    if start >= checkpoint {
                        retirement_spans.push((start, end));
                    } else if end > checkpoint {
                        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
                    }
                    return Ok(());
                }
                if super::super::super::retention::payload_is_retirement(frame.payload()) {
                    let Some(record) =
                        super::super::super::retention::decode_retirement(frame.payload())
                    else {
                        return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
                    };
                    if let Some(release) = record.release {
                        let entry = release_metadata
                            .entry(release.candidate_generation())
                            .or_insert((release, None));
                        if entry.0 != release {
                            return Err(PhysicalWalOpenFailure::MemberPayloadRejected);
                        }
                        if !record.completion && entry.1.is_none() {
                            entry.1 = Some((identity.segment().get(), identity.generation().get()));
                        }
                    }
                    retirement_records.push(record);
                    if let Some(intent) = super::RetainedMaintenanceIntent::from_verified(
                        artifact.clone(),
                        identity.segment().get(),
                        identity.generation().get(),
                        offset,
                        frame,
                    ) {
                        retained_maintenance.push(intent);
                    }
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
                        _ => retirement_spans
                            .push((range.start().get(), range.end_exclusive().get())),
                    }
                    return Ok(());
                }
                if worth_store_physical_format::payload_is_blob_manifest_residue_cleanup_any(
                    frame.payload(),
                ) {
                    manifest_residue::observe(
                        frame.payload(),
                        frame.lsn_range(),
                        media.store_identity().bytes(),
                        cutoff,
                        &mut retirement_spans,
                    )?;
                    return Ok(());
                }
                if worth_store_physical_format::payload_is_tier_epoch_activation(frame.payload()) {
                    tier_epoch::observe(
                        frame.payload(),
                        frame.lsn_range(),
                        media.store_identity().bytes(),
                        cutoff,
                        &mut retirement_spans,
                    )?;
                    return Ok(());
                }
                retained_released_drop |= publication_roster.observe(
                    frame,
                    identity.segment().get(),
                    identity.generation().get(),
                    record_format,
                    binding_context,
                    byte_count,
                    active_frame_views_bytes,
                )? && cutoff
                    .lsn()
                    .is_none_or(|lsn| frame.lsn_range().start() >= lsn);
                super::copy_publication::observe(
                    frame.payload(),
                    frame.lsn_range(),
                    record_format,
                    binding_context,
                    &mut copy_obligations,
                    cutoff.lsn().map_or(0, |lsn| lsn.get()),
                )
                .map_err(|_| PhysicalWalOpenFailure::MemberPayloadRejected)?;
                if let Some(member) =
                    ReopenedPhysicalWalMember::decode_retained_frame(cutoff, frame)
                        .map_err(|_denial| PhysicalWalOpenFailure::MemberPayloadRejected)?
                {
                    members.push(member);
                }
                Ok(())
            })();
            if let Err(failure) = semantic_observation {
                if failure == PhysicalWalOpenFailure::MemberPayloadRejected {
                    semantic_failure = Some(failure);
                } else {
                    return Err(failure);
                }
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
    if let Some(failure) = semantic_failure {
        return Err(failure);
    }
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
    let publication_groups = publication_roster.finish()?;
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
        record_format,
        copy_obligations,
        checkpoint_cutoff: cutoff
            .lsn()
            .unwrap_or(worth_store_wal::WAL_ORIGIN.lsn())
            .get(),
        frontier: WalAppendFrontier::observed(
            active.segment(),
            active.generation(),
            active_bytes,
            active_lsn_end,
        ),
        active_artifact,
        segment_count,
        frame_count: total_frames,
        publication_groups,
        release_metadata: release_metadata
            .into_values()
            .map(|(release, intent)| {
                let (bytes, segment, generation) = intent
                    .map_or((0, 0, 0), |(segment, generation)| {
                        (release.metadata_bytes(), segment, generation)
                    });
                (release.candidate_generation(), bytes, segment, generation)
            })
            .collect(),
        byte_count: total_bytes,
        peak_buffer_bytes,
        requires_inspection,
        release_evidence: super::RetainedWalReleaseEvidence::new(
            retained_released_drop,
            if cutoff.lsn().is_none() && !requires_inspection {
                super::RetainedWalHistory::FromOrigin
            } else {
                super::RetainedWalHistory::Suffix
            },
        ),
        segments: segment_inventory,
        members,
        retirement_spans,
        retirement_records,
        retirement_locations,
        retained_maintenance,
    })
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
