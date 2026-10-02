//! Independent Store check of the actual V3 source-to-result inventory. C.8's
//! checked token is compared only after Store rewalks both rooted snapshots.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedPhysicalRecoveryProjection, PhysicalInventoryTranscriptBuilderV1,
    PhysicalInventoryTranscriptV1, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    ReleasedInventoryView, VerifiedPendingWalReleaseCustody, VerifiedReleasedV3InventoryTransition,
};

use super::super::{
    control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint},
    tier, wal_fate, wal_inventory, SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_BYTES,
    MAX_DISCOVERY_ENTRIES,
};
use super::{
    controls::Controls, head_v14::ObservedHeadV14, selection, selection::Selection, topology,
};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, StoreRecoveryBindingFreshnessSample,
};

// Retained Store observations, both temporary inventories and the pure
// predicate's scratch share one admitted recovery-memory window. Charges are
// debited before an inventory reserve, not after both inventories exist.
pub(super) const MAX_TRANSITION_MEMORY: u64 = MAX_DISCOVERY_BYTES;
pub(super) const MAX_TRANSITION_ENTRIES: u64 =
    MAX_TRANSITION_MEMORY / (4 * std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64);
const DISCOVERY_HEADROOM: u64 = 64 << 20;
const SNAPSHOT_TREE_HEADROOM: u64 = 64 << 20;
// `observe_claim` drops the first WAL inventory before calling this budget.
// The final inventory owns at most MAX_WAL_BYTES of encoded frames; the extra
// allowance covers 65,536 frame structs, names, map nodes and its fingerprint.
const FINAL_WAL_INVENTORY_HEADROOM: u64 = wal_inventory::MAX_WAL_BYTES + (32 << 20);

pub(super) struct Snapshot {
    pub(super) routes: Vec<CurrentPhysicalRecordPlacement>,
    pub(super) segments: Vec<RecordSegmentPageManifestEntry>,
    pub(super) free_entries: Vec<RecordFreeSpaceManifestEntry>,
    pub(super) transcript: PhysicalInventoryTranscriptV1,
    pub(super) fingerprint: SelectedControlMediaFingerprint,
}

pub(super) fn verify(
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    prior: &Selection,
    controls: &Controls,
    observed_head: &ObservedHeadV14<'_>,
    claim: &VerifiedPendingWalReleaseCustody,
    projection: &PersistedPhysicalRecoveryProjection,
    sample: &StoreRecoveryBindingFreshnessSample,
    checkpoint: &crate::physical_runtime::SharedRecoveryCheckpoint,
) -> Result<AdmittedRecoveryFilesystemMedia, Denial> {
    let format = reopen.format();
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let selected = selection::observe(&mut discovery, reopen, claim, checkpoint.stream())?;
    if !prior.same_bytes(&selected) {
        return Err(Denial::RootBinding);
    }
    let (source_root, source_free, result_root, result_free) = (
        selected.source_root.clone(),
        selected.source_free.clone(),
        selected.root.clone(),
        selected.free.clone(),
    );
    drop(selected);
    let retained = retained_memory(prior, controls, claim, projection, sample, checkpoint)?;
    let mut remaining = MAX_TRANSITION_MEMORY
        .checked_sub(retained)
        .ok_or(Denial::BoundExceeded)?;
    let source = snapshot(
        &mut discovery,
        &source_root,
        &source_free,
        format,
        &mut remaining,
    )?;
    let result = snapshot(
        &mut discovery,
        &result_root,
        &result_free,
        format,
        &mut remaining,
    )?;
    let expected_transition = claim.verified_transition().ok_or(Denial::RoutingFrame)?;
    debit(
        &mut remaining,
        (expected_transition.projected().len() as u64).saturating_mul(
            4 * (std::mem::size_of::<worth_store_physical_format::PersistedRecordIdentity>()
                + 8 * std::mem::size_of::<usize>()) as u64,
        ),
    )?;
    if !wal_fate::matches_pending_projected_set(
        claim,
        sample,
        &source.routes,
        expected_transition.projected(),
        format,
    ) {
        return Err(Denial::WalFate);
    }
    let BlobRecordV1::DropSetManifestV3(manifest) =
        decode_blob_record(controls.manifest_bytes()).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let drop_count = manifest
        .dropped()
        .len()
        .saturating_add(match projection.operation() {
            worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
                retirement: Some(retirement),
                ..
            } => retirement.dropped_records().len(),
            _ => 0,
        });
    debit(
        &mut remaining,
        (drop_count as u64).saturating_mul(
            4 * std::mem::size_of::<worth_store_physical_format::PersistedRecordIdentity>() as u64,
        ),
    )?;
    let mut dropped = manifest.dropped().to_vec();
    if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = projection.operation()
    {
        dropped.extend_from_slice(retirement.dropped_records());
    }
    dropped.sort_unstable();
    if dropped.windows(2).any(|pair| pair[0] == pair[1])
        || !projection.segment_updates().is_empty()
        || !projection.root_state().inline_allocations().is_empty()
    {
        return Err(Denial::RoutingFrame);
    }
    let actual = VerifiedReleasedV3InventoryTransition::admit_with_head_replay(
        ReleasedInventoryView::new(
            &source_root,
            &source_free,
            &source.routes,
            &source.segments,
            &source.free_entries,
        ),
        ReleasedInventoryView::new(
            &result_root,
            &result_free,
            &result.routes,
            &result.segments,
            &result.free_entries,
        ),
        &dropped,
        expected_transition.projected(),
        observed_head.replay(),
        format,
        MAX_TRANSITION_ENTRIES,
        remaining,
    )
    .map_err(|_| Denial::RoutingFrame)?;
    let expected = claim.topologies().ok_or(Denial::RoutingFrame)?;
    if source.transcript != actual.source_topology()
        || result.transcript != actual.result_topology()
        || expected != (actual.source_topology(), actual.result_topology())
    {
        return Err(Denial::RoutingFrame);
    }
    Ok(discovery.finish())
}

pub(super) fn snapshot(
    discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
    root: &worth_store_physical_format::DurablePhysicalRootManifest,
    free: &worth_store_physical_format::DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    remaining: &mut u64,
) -> Result<Snapshot, Denial> {
    let maximum_segments = preflight_snapshot(root.record_count(), free.entry_count(), remaining)?;
    let mut transcript =
        PhysicalInventoryTranscriptBuilderV1::new(root, free, format, MAX_TRANSITION_ENTRIES)
            .map_err(|_| Denial::RoutingFrame)?;
    let (routes_proof, routes) =
        tier::routes::verify_snapshot(discovery, root, free, format, &mut transcript)?;
    let mut fingerprint = routes_proof.into_media_fingerprint();
    let (segments, free_entries, membership_slices) = topology::observe_membership_snapshot(
        discovery,
        root,
        free,
        format,
        &mut transcript,
        maximum_segments,
    )?;
    fingerprint.extend(SelectedControlMediaFingerprint::observed(membership_slices))?;
    for (artifact, bytes) in [
        (
            RecordArtifactFile::RootManifest {
                generation: root.generation(),
            },
            root.encode(format),
        ),
        (
            RecordArtifactFile::FreeSpaceManifest {
                generation: free.generation(),
            },
            free.encode(format),
        ),
    ] {
        fingerprint.extend(SelectedControlMediaFingerprint::observed(vec![
            SelectedArtifactSlice::observed(artifact, 0, &bytes, true)
                .ok_or(Denial::BoundExceeded)?,
        ]))?;
    }
    let segment_width = 4 * std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;
    debit(
        remaining,
        (segments.capacity() as u64).saturating_mul(segment_width),
    )?;
    let transcript = transcript.finish().map_err(|_| Denial::RoutingFrame)?;
    Ok(Snapshot {
        routes,
        segments,
        free_entries,
        transcript,
        fingerprint,
    })
}

fn preflight_snapshot(
    route_count: u64,
    free_count: u64,
    remaining: &mut u64,
) -> Result<u64, Denial> {
    // The fixed allowance covers route/membership traversal stacks, visited
    // node sets and compact block fingerprints (each capped at 65,536 nodes).
    debit(remaining, SNAPSHOT_TREE_HEADROOM)?;
    debit(
        remaining,
        route_count
            .checked_mul(4 * std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64)
            .ok_or(Denial::BoundExceeded)?,
    )?;
    debit(
        remaining,
        free_count
            .checked_mul(4 * std::mem::size_of::<RecordFreeSpaceManifestEntry>() as u64)
            .ok_or(Denial::BoundExceeded)?,
    )?;
    let segment_width = 4 * std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;
    Ok(*remaining / (2 * segment_width))
}

fn debit(remaining: &mut u64, bytes: u64) -> Result<(), Denial> {
    *remaining = remaining.checked_sub(bytes).ok_or(Denial::BoundExceeded)?;
    Ok(())
}

pub(super) fn retained_memory(
    selected: &Selection,
    controls: &Controls,
    claim: &VerifiedPendingWalReleaseCustody,
    projection: &PersistedPhysicalRecoveryProjection,
    sample: &StoreRecoveryBindingFreshnessSample,
    checkpoint: &crate::physical_runtime::SharedRecoveryCheckpoint,
) -> Result<u64, Denial> {
    let bytes = selected
        .retained_memory_bytes()
        .saturating_add(controls.retained_memory_bytes())
        .saturating_add(claim_memory(claim))
        .saturating_add(checkpoint.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
        .saturating_add(projection_memory(projection))
        .saturating_add(sample_memory(sample)?)
        .saturating_add(DISCOVERY_HEADROOM)
        .saturating_add(FINAL_WAL_INVENTORY_HEADROOM)
        .saturating_add(8 << 20);
    (bytes < MAX_TRANSITION_MEMORY)
        .then_some(bytes)
        .ok_or(Denial::BoundExceeded)
}

fn claim_memory(claim: &VerifiedPendingWalReleaseCustody) -> u64 {
    let batches = claim
        .selected_release()
        .map_or(0, |base| 4 * std::mem::size_of_val(base.batches()) as u64);
    let projected = claim.verified_transition().map_or(0, |transition| {
        4 * std::mem::size_of_val(transition.projected()) as u64
    });
    let historical = claim.historical_batches().iter().fold(0_u64, |sum, batch| {
        let chain = batch.chain();
        let prefix = chain
            .checkpoint_prefix()
            .map_or(0, |prefix| 4 * std::mem::size_of_val(prefix.steps()) as u64);
        sum.saturating_add(4 * std::mem::size_of_val(batch) as u64)
            .saturating_add(4 * std::mem::size_of_val(chain.ordinary_steps()) as u64)
            .saturating_add(prefix)
            .saturating_add(4 * std::mem::size_of_val(chain.first_transition().projected()) as u64)
            .saturating_add(4 * std::mem::size_of_val(batch.manifest().dropped()) as u64)
    });
    let ordered = claim.ordered_history().map_or(0, |history| {
        let edges = history.edges().iter().fold(0_u64, |sum, edge| {
            let projected = match edge {
                worth_store_recovery_physics::VerifiedOrderedRootEdge::Ordinary(_) => 0,
                worth_store_recovery_physics::VerifiedOrderedRootEdge::Released(release) => {
                    4 * std::mem::size_of_val(release.transition().projected()) as u64
                }
            };
            sum.saturating_add(4 * std::mem::size_of_val(edge) as u64)
                .saturating_add(projected)
        });
        edges.saturating_add(
            claim
                .ordered_released_batches()
                .iter()
                .fold(0_u64, |sum, batch| {
                    sum.saturating_add(4 * batch.retained_bytes())
                }),
        )
    });
    (std::mem::size_of_val(claim) as u64)
        .saturating_add(batches)
        .saturating_add(projected)
        .saturating_add(historical)
        .saturating_add(ordered)
}

pub(super) fn projection_memory(projection: &PersistedPhysicalRecoveryProjection) -> u64 {
    let mut bytes = (std::mem::size_of_val(projection) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.record_identities()) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.placements()) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.segment_updates()) as u64)
        .saturating_add(4 * std::mem::size_of_val(projection.manifests()) as u64)
        .saturating_add(
            4 * std::mem::size_of_val(projection.root_state().inline_allocations()) as u64,
        );
    if let Some(frames) = projection.frames() {
        bytes = bytes.saturating_add(4 * std::mem::size_of_val(frames) as u64);
        for frame in frames {
            bytes = bytes.saturating_add(frame.bytes().len() as u64);
        }
    }
    for manifest in projection.manifests() {
        bytes = bytes.saturating_add(manifest.bytes().len() as u64);
    }
    if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = projection.operation()
    {
        bytes =
            bytes.saturating_add(4 * std::mem::size_of_val(retirement.dropped_records()) as u64);
    }
    bytes
}

pub(super) fn sample_memory(sample: &StoreRecoveryBindingFreshnessSample) -> Result<u64, Denial> {
    // Preserve the transition envelope's 4x roster / 1x payload allowance,
    // using actual retained capacities rather than visible slice lengths.
    let heap = sample.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
    let roster = sample.roster_heap_bytes().ok_or(Denial::BoundExceeded)?;
    roster
        .checked_mul(3)
        .and_then(|extra| heap.checked_add(extra))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of_val(sample) as u64))
        .ok_or(Denial::BoundExceeded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_inventory_is_denied_before_any_snapshot_allocation() {
        let route_width = 4 * std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64;
        let free_width = 4 * std::mem::size_of::<RecordFreeSpaceManifestEntry>() as u64;
        let one = SNAPSHOT_TREE_HEADROOM + route_width + free_width;
        let mut remaining = one * 2 - 1;
        preflight_snapshot(1, 1, &mut remaining).unwrap();
        assert!(matches!(
            preflight_snapshot(1, 1, &mut remaining),
            Err(Denial::BoundExceeded)
        ));
    }
}
