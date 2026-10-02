//! A completed V3 edge reuses Store's actual-media delta predicate, with
//! native-backed snapshots and control reads before the head-fold callback.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    PersistedPhysicalRecoveryOperation, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    ReleasedInventoryView, VerifiedOrderedPendingWalReleaseBatch,
    VerifiedOrderedReleasedHeadReplayV14, VerifiedReleasedRootEdge,
    VerifiedReleasedV3InventoryTransition,
};

use super::super::super::{
    completed_history::{
        native_storage::HistoricalWalkStorage, CompletedHistoryHeadFold, CompletedHistoryHeadStep,
    },
    control_frames::{
        FundedCompletedHistoricalRawSlices, SelectedArtifactSlice, SelectedControlMediaFingerprint,
    },
    resident::StoreRejoinResidentLedger,
    SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::super::{addressed_root, delta, head_effect_media};
use super::{controls, member_projection, verify_delta_with_storage};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, PhysicalRecoveryReadAllocation,
    StoreRecoveryBindingFreshnessSample,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_completed(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    raw: &mut FundedCompletedHistoricalRawSlices,
    fold: &mut CompletedHistoryHeadFold,
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    replay: &VerifiedOrderedReleasedHeadReplayV14,
    generation: u64,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let projection = member_projection::matched_projection_with_storage(
        edge, batch, replay, generation, sample, frames, cutoff, format, window, resident,
    )?;
    projection.with_data(resident, |projection, resident| {
        let mut discovery = media
            .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
            .map_err(Denial::Qualification)?;
        let mut storage = HistoricalWalkStorage::new(window, resident, raw);
        let source = addressed_root::observe_with_storage(
            &mut discovery,
            generation,
            node_capacity,
            format,
            edge.transition().source_topology(),
            Some(edge.source_root_frame_sha256()),
            &mut storage,
        )?;
        if source.free_sha256 != edge.source_free_space_frame_sha256() {
            return Err(Denial::RootBinding);
        }
        let result = addressed_root::observe_with_storage(
            &mut discovery,
            generation.checked_add(1).ok_or(Denial::BoundExceeded)?,
            node_capacity,
            format,
            edge.transition().result_topology(),
            Some(edge.result_root_frame_sha256()),
            &mut storage,
        )?;
        let source_snapshot = delta::snapshot_with_storage(
            &mut discovery,
            &source.root,
            &source.free,
            format,
            &mut storage,
        )?;
        let result_snapshot = delta::snapshot_with_storage(
            &mut discovery,
            &result.root,
            &result.free,
            format,
            &mut storage,
        )?;
        if source_snapshot.transcript != edge.transition().source_topology()
            || result_snapshot.transcript != edge.transition().result_topology()
        {
            return Err(Denial::RoutingFrame);
        }
        drop(storage);
        let effect = replay.replay().effect();
        let scratch = effect
            .verification_additional_peak_bytes(format)
            .ok_or(Denial::BoundExceeded)?;
        let count = effect
            .source_path()
            .len()
            .checked_add(effect.node_writes().len())
            .ok_or(Denial::BoundExceeded)?;
        let retained = (count as u64)
            .checked_mul(std::mem::size_of::<SelectedArtifactSlice>() as u64)
            .ok_or(Denial::BoundExceeded)?;
        resident
            .transient(
                scratch
                    .checked_add(retained)
                    .and_then(|bytes| bytes.checked_add(u64::from(format.page_size().bytes())))
                    .ok_or(Denial::BoundExceeded)?,
            )
            .map_err(Denial::Resident)?;
        let observed_head = head_effect_media::observe(
            &mut discovery,
            window,
            replay.replay(),
            &source.root,
            &result.root,
            format,
            resident.remaining(),
        )?;
        let head_bytes = observed_head
            .owned_heap_bytes()
            .ok_or(Denial::BoundExceeded)?;
        resident.retain(head_bytes).map_err(Denial::Resident)?;
        let mut storage = HistoricalWalkStorage::new(window, resident, raw);
        let controls = controls::verify_with_storage(
            &mut discovery,
            &source_snapshot,
            &result_snapshot,
            edge,
            batch,
            format,
            &mut storage,
        )?;
        let source_view = ReleasedInventoryView::new(
            &source.root,
            &source.free,
            &source_snapshot.routes,
            &source_snapshot.segments,
            &source_snapshot.free_entries,
        );
        let result_view = ReleasedInventoryView::new(
            &result.root,
            &result.free,
            &result_snapshot.routes,
            &result_snapshot.segments,
            &result_snapshot.free_entries,
        );
        let proof_peak = VerifiedReleasedV3InventoryTransition::maximum_construction_heap_bytes(
            source_view,
            result_view,
            projection.placements(),
            delta::MAX_TRANSITION_ENTRIES,
        )
        .ok_or(Denial::BoundExceeded)?;
        storage
            .resident
            .transient(proof_peak)
            .map_err(Denial::Resident)?;
        let grant = storage
            .window
            .reserve_owned(proof_peak)
            .map_err(Denial::Resident)?;
        storage
            .resident
            .retain(proof_peak)
            .map_err(Denial::Resident)?;
        let proof = verify_delta_with_storage(
            &source,
            &result,
            &source_snapshot,
            &result_snapshot,
            edge,
            batch,
            projection,
            observed_head.replay(),
            format,
            delta::MAX_TRANSITION_MEMORY,
            &mut storage,
        );
        drop(grant);
        storage.resident.release(proof_peak);
        proof?;
        let PersistedPhysicalRecoveryOperation::RecordsDropped {
            head_effect: Some(actual_effect),
            ..
        } = projection.operation()
        else {
            return Err(Denial::WalFate);
        };
        fold.apply(CompletedHistoryHeadStep::Released {
            effect: actual_effect,
            controls: controls.head_controls()?,
            source: &source.root,
            result: &result.root,
        })?;
        let mut fingerprint = source.fingerprint;
        fingerprint.extend_with_storage(result.fingerprint, &mut storage)?;
        fingerprint.extend_with_storage(source_snapshot.fingerprint, &mut storage)?;
        fingerprint.extend_with_storage(result_snapshot.fingerprint, &mut storage)?;
        fingerprint.extend_with_storage(observed_head.into_fingerprint(), &mut storage)?;
        fingerprint.extend_with_storage(controls.into_fingerprint(&mut storage)?, &mut storage)?;
        storage.discard_vec(source_snapshot.routes)?;
        storage.discard_vec(source_snapshot.segments)?;
        storage.discard_vec(source_snapshot.free_entries)?;
        storage.discard_vec(result_snapshot.routes)?;
        storage.discard_vec(result_snapshot.segments)?;
        storage.discard_vec(result_snapshot.free_entries)?;
        Ok((discovery.finish(), fingerprint))
    })
}
