//! One completed ordinary edge: funded Store reads, the existing Physics
//! actual-media predicate, and an unchanged-head step in the Store fold.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{PhysicalInventoryTranscriptV1, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{ReleasedInventoryView, VerifiedOrdinaryRootStep};

use super::super::super::{
    completed_history::{
        native_storage::HistoricalWalkStorage, CompletedHistoryHeadFold, CompletedHistoryHeadStep,
    },
    control_frames::{FundedCompletedHistoricalRawSlices, SelectedControlMediaFingerprint},
    resident::StoreRejoinResidentLedger,
    SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::super::{addressed_root, delta, ordinary_member};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, PhysicalRecoveryReadAllocation,
    StoreRecoveryBindingFreshnessSample,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_completed_ordinary_step(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    raw: &mut FundedCompletedHistoricalRawSlices,
    fold: &mut CompletedHistoryHeadFold,
    step: &VerifiedOrdinaryRootStep,
    generation: u64,
    topology: PhysicalInventoryTranscriptV1,
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
    let range = step.lsn_range().ok_or(Denial::WalFate)?;
    if step.source_topology() != topology || range.start().get() < cutoff {
        return Err(Denial::WalFate);
    }
    let matched = ordinary_member::match_step_with_storage(
        step, sample, frames, cutoff, format, window, resident,
    )?;
    matched.with_data(resident, |matched, resident| {
        let mut discovery = media
            .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
            .map_err(Denial::Qualification)?;
        let mut storage = HistoricalWalkStorage::new(window, resident, raw);
        let source = addressed_root::observe_with_storage(
            &mut discovery,
            generation,
            node_capacity,
            format,
            step.source_topology(),
            None,
            &mut storage,
        )?;
        let result = addressed_root::observe_with_storage(
            &mut discovery,
            generation.checked_add(1).ok_or(Denial::BoundExceeded)?,
            node_capacity,
            format,
            step.result_topology(),
            None,
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
        if source_snapshot.transcript != step.source_topology()
            || result_snapshot.transcript != step.result_topology()
        {
            return Err(Denial::RoutingFrame);
        }
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
        let required = VerifiedOrdinaryRootStep::maximum_recheck_heap_bytes(
            source_view,
            result_view,
            matched.observed().projection,
            delta::MAX_TRANSITION_ENTRIES,
        )
        .ok_or(Denial::BoundExceeded)?;
        storage
            .resident
            .transient(required)
            .map_err(Denial::Resident)?;
        drop(storage);
        let grant = window.reserve_owned(required).map_err(Denial::Resident)?;
        resident.retain(required).map_err(Denial::Resident)?;
        let rechecked = step.recheck_actual_media(
            source_view,
            result_view,
            matched.observed(),
            format,
            delta::MAX_TRANSITION_ENTRIES,
            required,
        );
        drop(grant);
        resident.release(required);
        rechecked.map_err(|_| Denial::RoutingFrame)?;
        fold.apply(CompletedHistoryHeadStep::Ordinary {
            source: &source.root,
            result: &result.root,
        })?;
        let mut storage = HistoricalWalkStorage::new(window, resident, raw);
        let mut fingerprint = source.fingerprint;
        fingerprint.extend_with_storage(result.fingerprint, &mut storage)?;
        fingerprint.extend_with_storage(source_snapshot.fingerprint, &mut storage)?;
        fingerprint.extend_with_storage(result_snapshot.fingerprint, &mut storage)?;
        storage.discard_vec(source_snapshot.routes)?;
        storage.discard_vec(source_snapshot.segments)?;
        storage.discard_vec(source_snapshot.free_entries)?;
        storage.discard_vec(result_snapshot.routes)?;
        storage.discard_vec(result_snapshot.segments)?;
        storage.discard_vec(result_snapshot.free_entries)?;
        Ok((discovery.finish(), fingerprint))
    })
}
