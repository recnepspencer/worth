use std::collections::HashSet;
use std::sync::Arc;

use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, FreeSpaceBlockReference,
    ManifestBlockReference, SegmentManifestBlockReference,
};

use super::{damaged, metadata_pressure};
use crate::physical_runtime::record_serving::{
    access::{
        manifest_routing::{
            ManifestDiscoveryCounterSnapshot, ManifestLookupFailure, ManifestReader,
        },
        segment_membership::SegmentMembershipReader,
    },
    admission::{
        bootstrap::{BootstrapTransitionFailure, RecordBootstrapDenial},
        open::CurrentRootAdmission,
    },
    planning::free_space_routing::FreeSpaceReader,
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy,
};

#[derive(Clone, Copy)]
enum RoutingReference {
    Record(ManifestBlockReference),
    Segment(SegmentManifestBlockReference),
    FreeSpace(FreeSpaceBlockReference),
}

impl RoutingReference {
    fn generation(self) -> u64 {
        match self {
            Self::Record(reference) => reference.generation(),
            Self::Segment(reference) => reference.generation(),
            Self::FreeSpace(reference) => reference.generation(),
        }
    }

    fn identity(self) -> (u8, u64) {
        match self {
            Self::Record(reference) => (0, reference.block()),
            Self::Segment(reference) => (1, reference.block()),
            Self::FreeSpace(reference) => (2, reference.block()),
        }
    }
}

/// Count only blocks first written by this root. Older references name immutable
/// shared blocks and cannot hide a newly emitted child beneath themselves.
pub(super) fn new_generation_bytes(
    admission: &CurrentRootAdmission<'_>,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    root: &DurablePhysicalRootManifest,
    free_space: &DurableFreeSpaceManifestHeader,
) -> Result<u64, BootstrapTransitionFailure> {
    let record_reader = ManifestReader::with_loader(
        admission.media,
        admission.loader,
        format,
        access,
        root,
        Arc::clone(&admission.lifecycle),
        admission.resident_integrity_counters,
    );
    let segment_reader = SegmentMembershipReader::with_loader(
        admission.media,
        admission.loader,
        format,
        access,
        root,
        Arc::clone(&admission.lifecycle),
        admission.resident_integrity_counters,
    );
    let free_reader = FreeSpaceReader::with_loader(
        admission.media,
        admission.loader,
        format,
        access,
        free_space,
        Arc::clone(&admission.lifecycle),
        admission.resident_integrity_counters,
    );
    let max_blocks = routing_scratch_limit(
        admission.allocation.bytes(),
        u64::from(format.declaration().page_size().bytes()),
    )?;
    let mut pending = Vec::new();
    let mut seen = HashSet::new();
    for reference in root
        .routing_root()
        .map(RoutingReference::Record)
        .into_iter()
        .chain(root.segment_root().map(RoutingReference::Segment))
        .chain(free_space.root().map(RoutingReference::FreeSpace))
    {
        push_new(
            reference,
            root.generation(),
            max_blocks,
            &seen,
            &mut pending,
        )?;
    }
    let mut counters = ManifestDiscoveryCounterSnapshot::default();
    let mut total = 0_u64;
    while let Some(reference) = pending.pop() {
        if seen.len() as u64 >= max_blocks {
            return Err(metadata_pressure());
        }
        seen.try_reserve(1).map_err(|_| metadata_pressure())?;
        if !seen.insert(reference.identity()) {
            return Err(damaged());
        }
        let bytes = match reference {
            RoutingReference::Record(reference) => {
                let (block, bytes) = record_reader
                    .read_block_with_len(admission.allocation, reference, &mut counters)
                    .map_err(classify_block)?;
                if let Some(children) = block.children() {
                    for child in children {
                        if reference.level().checked_sub(1) != Some(child.level()) {
                            return Err(damaged());
                        }
                        push_new(
                            RoutingReference::Record(*child),
                            root.generation(),
                            max_blocks,
                            &seen,
                            &mut pending,
                        )?;
                    }
                }
                bytes
            }
            RoutingReference::Segment(reference) => {
                let (block, bytes) = segment_reader
                    .read_block_with_len(admission.allocation, reference, &mut counters)
                    .map_err(classify_block)?;
                if let Some(children) = block.children() {
                    for child in children {
                        if reference.level().checked_sub(1) != Some(child.level()) {
                            return Err(damaged());
                        }
                        push_new(
                            RoutingReference::Segment(*child),
                            root.generation(),
                            max_blocks,
                            &seen,
                            &mut pending,
                        )?;
                    }
                }
                bytes
            }
            RoutingReference::FreeSpace(reference) => {
                let (block, bytes) = free_reader
                    .read_block_with_len(admission.allocation, reference, &mut counters)
                    .map_err(classify_block)?;
                if let Some(children) = block.children() {
                    for child in children {
                        if reference.level().checked_sub(1) != Some(child.level()) {
                            return Err(damaged());
                        }
                        push_new(
                            RoutingReference::FreeSpace(*child),
                            root.generation(),
                            max_blocks,
                            &seen,
                            &mut pending,
                        )?;
                    }
                }
                bytes
            }
        };
        total = total.checked_add(bytes).ok_or_else(damaged)?;
    }
    Ok(total)
}

fn push_new(
    reference: RoutingReference,
    generation: u64,
    max_blocks: u64,
    seen: &HashSet<(u8, u64)>,
    pending: &mut Vec<RoutingReference>,
) -> Result<(), BootstrapTransitionFailure> {
    if reference.generation() > generation {
        return Err(damaged());
    }
    if reference.generation() < generation {
        return Ok(());
    }
    let occupied = (seen.len() as u64)
        .checked_add(pending.len() as u64)
        .ok_or_else(metadata_pressure)?;
    if occupied >= max_blocks {
        return Err(metadata_pressure());
    }
    pending.try_reserve(1).map_err(|_| metadata_pressure())?;
    pending.push(reference);
    Ok(())
}

/// Includes two Vec capacities, four hash-table slots per key, and a fixed
/// per-entry allocator allowance; three resident frames remain available for
/// the concurrently held record, segment, and free-space decoders.
fn routing_scratch_limit(
    grant_bytes: u64,
    page_bytes: u64,
) -> Result<u64, BootstrapTransitionFailure> {
    let frame_headroom = page_bytes.checked_mul(3).ok_or_else(metadata_pressure)?;
    let scratch_bytes = grant_bytes
        .checked_sub(frame_headroom)
        .ok_or_else(metadata_pressure)?;
    let per_reference = (std::mem::size_of::<RoutingReference>() as u64)
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add((std::mem::size_of::<(u8, u64)>() as u64 + 1) * 4))
        .and_then(|bytes| bytes.checked_add(64))
        .ok_or_else(metadata_pressure)?;
    let limit = scratch_bytes / per_reference;
    (limit > 0).then_some(limit).ok_or_else(metadata_pressure)
}

#[cfg(test)]
mod tests {
    use super::routing_scratch_limit;

    #[test]
    fn routing_reopen_denies_when_frames_leave_no_scratch_for_one_reference() {
        let frames = 3 * 16 * 1024;
        assert!(routing_scratch_limit(frames, 16 * 1024).is_err());
        assert!(matches!(
            routing_scratch_limit(frames + 1024, 16 * 1024),
            Ok(limit) if limit >= 1
        ));
    }
}

fn classify_block(failure: ManifestLookupFailure) -> BootstrapTransitionFailure {
    match failure {
        ManifestLookupFailure::Backend(failure) => {
            super::super::bootstrap::backend_before_effect(failure)
        }
        ManifestLookupFailure::Residency(reason) => {
            BootstrapTransitionFailure::Denied(RecordBootstrapDenial::from_residency(reason))
        }
        _ => damaged(),
    }
}
