use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, FailedIngestReclaimBasisV1, PersistedRecordIdentity,
};

use crate::physical_runtime::{
    durability::{CompletedDurableCheckpointWitness, PhysicalRecoveredOriginalDropNoDurableEffect},
    AdmittedBlobScope, AdmittedRecordPlacementPolicy, BlobPhysicalAllocation, BlobResumeToken,
    PhysicalRecordReader, ServingPhysicalRuntime,
};

use super::super::ingest::selected_session::read_authenticated_declaration;
use super::{scan, BlobReclaimFailure, BlobReclaimLimits};

mod custody;
use custody::observe_custody;
pub(super) use custody::ResidueCandidate;
mod edges;
use edges::{mark_incoming_edges, protect_frontier_prefix};
mod reservation;
use reservation::{
    reservation_blocks_fresh_attempt, SelectionDescriptorLink, SelectionReservationLink,
};

pub(super) const SELECTION_ROSTER_ENTRY_BYTES: usize =
    std::mem::size_of::<SelectionDescriptorLink>()
        + std::mem::size_of::<SelectionReservationLink>()
        + std::mem::size_of::<PhysicalRecoveredOriginalDropNoDurableEffect>();

/// Selected custody and graph facts, never a caller-created orphan list.
pub(in crate::physical_runtime) struct SelectedFailedBlobResidue {
    reader: PhysicalRecordReader,
    basis: FailedIngestReclaimBasisV1,
    dropped: Vec<PersistedRecordIdentity>,
    remaining: u64,
    occupied_attempts: Vec<[u8; 16]>,
    recovered_reservations: Vec<PhysicalRecoveredOriginalDropNoDurableEffect>,
    inspection: scan::ReclaimInspectionWork,
}

impl SelectedFailedBlobResidue {
    pub(in crate::physical_runtime) fn into_reclaim_parts(
        self,
    ) -> (
        PhysicalRecordReader,
        FailedIngestReclaimBasisV1,
        Vec<PersistedRecordIdentity>,
        u64,
        Vec<[u8; 16]>,
        Vec<PhysicalRecoveredOriginalDropNoDurableEffect>,
    ) {
        (
            self.reader,
            self.basis,
            self.dropped,
            self.remaining,
            self.occupied_attempts,
            self.recovered_reservations,
        )
    }

    pub(super) fn is_empty(&self) -> bool {
        self.dropped.is_empty()
    }
    pub(super) fn remaining(&self) -> u64 {
        self.remaining
    }
    pub(super) fn inspection(&self) -> scan::ReclaimInspectionWork {
        self.inspection
    }
}

pub(super) fn select(
    runtime: &ServingPhysicalRuntime,
    reader: PhysicalRecordReader,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
    limits: BlobReclaimLimits,
    placement: AdmittedRecordPlacementPolicy,
    completed: Option<CompletedDurableCheckpointWitness>,
    allocation: &BlobPhysicalAllocation<'_>,
) -> Result<SelectedFailedBlobResidue, BlobReclaimFailure> {
    let required = limits
        .memory_bytes()
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if allocation.bytes() < required.get() {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let reader = reader.into_rebuild();
    let mut work = scan::ReclaimInspectionWork::default();
    scan::admit_inspected_bytes(
        &mut work,
        limits,
        super::super::ingest::selected_session::DECLARATION_FRAME_BYTES,
    )?;
    let declaration = read_authenticated_declaration(&reader, &token, scope)?;
    work.records = 1;
    let capacity = usize::try_from(limits.maximum_selected_records())
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    let mut candidates = Vec::new();
    candidates
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if candidates.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut occupied_attempts = Vec::new();
    occupied_attempts
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if occupied_attempts.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut scratch = Vec::new();
    scratch
        .try_reserve_exact(scan::FRAME_WINDOW_BYTES)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    scratch.resize(scan::FRAME_WINDOW_BYTES, 0);
    if scratch.capacity() != scan::FRAME_WINDOW_BYTES {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut declaration_seen = false;
    let mut abandoned = None;
    let reader = scan::walk(reader, limits, &mut scratch, &mut work, |record, bytes| {
        observe_custody(
            record,
            bytes,
            token,
            declaration,
            completed,
            &mut declaration_seen,
            &mut abandoned,
            &mut candidates,
        )
    })?;
    if !declaration_seen {
        return Err(BlobReclaimFailure::DeclarationMismatch);
    }
    let (abandoned_record, abandoned_digest) = abandoned.ok_or(BlobReclaimFailure::NotAbandoned)?;
    let basis = FailedIngestReclaimBasisV1::new(
        token.session,
        token.declaration_record,
        token.declaration_digest,
        abandoned_record,
        abandoned_digest,
    )
    .map_err(BlobReclaimFailure::Format)?;
    candidates.sort_unstable_by_key(|candidate| candidate.record);
    let mut frontier_prefix = 0;
    let basis_digest = basis.digest(token.store);
    let mut reservation_links = Vec::new();
    reservation_links
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if reservation_links.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let mut descriptor_links = Vec::new();
    descriptor_links
        .try_reserve_exact(capacity)
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if descriptor_links.capacity() != capacity {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    let reader = scan::walk(reader, limits, &mut scratch, &mut work, |record, bytes| {
        if bytes.starts_with(b"WRC11BLB") {
            match decode_blob_record(bytes).map_err(BlobReclaimFailure::Format)? {
                BlobRecordV1::OriginalDropReserved(reserved)
                    if reservation_blocks_fresh_attempt(reserved, token.store, basis_digest) =>
                {
                    reservation_links.push(SelectionReservationLink {
                        record,
                        frame_sha256: Sha256::digest(bytes).into(),
                        reserved,
                        matched_manifest_count: None,
                    });
                }
                BlobRecordV1::ReclaimDescriptor(descriptor)
                    if descriptor.store() == token.store
                        && descriptor.source_basis_digest() == basis_digest =>
                {
                    descriptor_links.push(SelectionDescriptorLink { record, descriptor });
                }
                _ => {}
            }
        }
        mark_incoming_edges(
            bytes,
            token.session,
            &mut candidates,
            &mut frontier_prefix,
            &mut occupied_attempts,
        )
    })?;
    occupied_attempts.sort_unstable();
    occupied_attempts.dedup();
    protect_frontier_prefix(&mut candidates, frontier_prefix);
    // Only current graph roots are removed in one batch. Selected parents keep
    // their children alive even within the failed session. Later bounded calls
    // can remove those children without ever leaving dangling selected edges.
    let mut dropped = Vec::new();
    dropped
        .try_reserve_exact(usize::from(limits.maximum_dropped_records()))
        .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
    if dropped.capacity() != usize::from(limits.maximum_dropped_records()) {
        return Err(BlobReclaimFailure::ScratchUnavailable);
    }
    for candidate in &candidates {
        if !candidate.referenced && dropped.len() < usize::from(limits.maximum_dropped_records()) {
            dropped.push(candidate.record);
        }
    }
    // A selected reservation may open a fresh attempt only when its original
    // descriptor has an exact, registry-sealed terminal no-effect. Empty
    // payload takes the manifest-only proof path instead.
    let (reader, recovered_reservations) = if !reservation_links.is_empty() && !dropped.is_empty() {
        reservation::verify_terminal_history(
            runtime,
            reader,
            token.store,
            basis,
            &mut reservation_links,
            &mut descriptor_links,
            limits,
            placement,
            &mut scratch,
            &mut work,
        )?
    } else {
        (reader, Vec::new())
    };
    let remaining = (candidates.len() - dropped.len()) as u64;
    Ok(SelectedFailedBlobResidue {
        reader,
        basis,
        dropped,
        remaining,
        occupied_attempts,
        recovered_reservations,
        inspection: work,
    })
}

#[cfg(test)]
#[path = "selection/frontier_tests.rs"]
mod frontier_tests;

#[cfg(test)]
#[path = "selection/reservation_tests.rs"]
mod reservation_tests;
