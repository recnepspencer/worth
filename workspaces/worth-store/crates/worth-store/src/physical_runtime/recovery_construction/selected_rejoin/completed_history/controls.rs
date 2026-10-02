//! Selected-tip control closure for the completed-history NoRelease base.
//! Released controls belong to ordered released batches; independently
//! validated ordinary-ingest controls retain their own actual selected bytes.
//! Failed-ingest controls use the same source join as NoRelease, with native
//! backing for their selected frames and temporary proofs.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, SelectedRecordContentClass,
};
use worth_store_recovery_physics::VerifiedOrderedHistoricalReleaseCustody;

use super::super::{
    control_frames::{
        read_extent_with_storage, FundedCompletedHistoricalRawSlices, SelectedArtifactSlice,
    },
    resident::StoreRejoinResidentLedger,
    tier, SelectedControlMediaFingerprint, SelectedMediaRejoinDenial as Denial,
    MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::{native_storage::HistoricalWalkStorage, selection::Selection};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, PhysicalRecoveryReadAllocation,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn observe(
    media: AdmittedRecoveryFilesystemMedia,
    selected: &Selection,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
    raw: &mut FundedCompletedHistoricalRawSlices,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let reread =
        super::selection::observe(&mut discovery, reopen, claim, checkpoint, window, resident)?;
    if !reread.same_bytes(selected) {
        return Err(Denial::RootBinding);
    }
    reread.discard(resident)?;
    let mut storage = HistoricalWalkStorage::new(window, resident, raw);
    let routes = tier::routes::verify_with_storage(
        &mut discovery,
        &selected.manifest,
        &selected.free_header,
        reopen.format(),
        &mut storage,
    )?;
    let SelectedControlPartition {
        failed_routes,
        mut ordinary_slices,
    } = observe_selected_control_partition(
        &mut discovery,
        routes.selected_routes(),
        claim,
        reopen.format(),
        &mut storage,
    )?;
    let proven = tier::no_release_controls::verify_with_storage(
        &mut discovery,
        failed_routes.as_slice(),
        reopen.format(),
        selected.manifest.generation(),
        claim.checkpoint().source().identity().sequence().get(),
        &mut ordinary_slices,
        usize::MAX,
        &mut storage,
    )?;
    storage.discard_vec(proven)?;
    storage.discard_vec(failed_routes)?;
    let mut fingerprint = routes.into_media_fingerprint_with_storage(&mut storage)?;
    fingerprint.extend_with_storage(
        SelectedControlMediaFingerprint::observed(ordinary_slices),
        &mut storage,
    )?;
    drop(storage);
    Ok((discovery.finish(), fingerprint))
}

struct SelectedControlPartition {
    failed_routes: Vec<CurrentPhysicalRecordPlacement>,
    ordinary_slices: Vec<SelectedArtifactSlice>,
}

fn observe_selected_control_partition(
    discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
    routes: &[CurrentPhysicalRecordPlacement],
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    storage: &mut HistoricalWalkStorage<'_, '_>,
) -> Result<SelectedControlPartition, Denial> {
    let count = claim
        .released_batches()
        .len()
        .checked_mul(3)
        .ok_or(Denial::BoundExceeded)?;
    let mut admitted = storage.reserve_vec::<PersistedRecordIdentity>(count)?;
    for batch in claim.released_batches() {
        admitted.extend([
            batch.descriptor_frame().record(),
            batch.reservation_frame().record(),
            batch.manifest_frame().record(),
        ]);
    }
    admitted.sort_unstable();
    if admitted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Denial::CertificateRoster);
    }
    for route in routes {
        if matches!(
            classify_selected_control(*route, &admitted),
            SelectedControlRole::Unsupported
        ) {
            return Err(Denial::UnsupportedSelectedControl {
                record: route.record(),
                content_class: route.content_class(),
            });
        }
    }
    let failed_count = routes
        .iter()
        .filter(|route| {
            matches!(
                classify_selected_control(**route, &admitted),
                SelectedControlRole::Ordinary | SelectedControlRole::FailedIngest,
            )
        })
        .count();
    let mut failed_routes = storage.reserve_vec::<CurrentPhysicalRecordPlacement>(failed_count)?;
    let mut ordinary_slices = storage.reserve_vec::<SelectedArtifactSlice>(0)?;
    for route in routes {
        match classify_selected_control(*route, &admitted) {
            SelectedControlRole::Released | SelectedControlRole::Unrelated => {}
            SelectedControlRole::Ordinary => {
                verify_ordinary_ingest_control(
                    discovery,
                    *route,
                    format,
                    &mut ordinary_slices,
                    storage,
                )?;
                failed_routes.push(*route);
            }
            SelectedControlRole::FailedIngest => failed_routes.push(*route),
            SelectedControlRole::Unsupported => {
                return Err(Denial::UnsupportedSelectedControl {
                    record: route.record(),
                    content_class: route.content_class(),
                });
            }
        }
    }
    storage.discard_vec(admitted)?;
    Ok(SelectedControlPartition {
        failed_routes,
        ordinary_slices,
    })
}

#[derive(Clone, Copy)]
enum SelectedControlRole {
    Released,
    Ordinary,
    FailedIngest,
    Unsupported,
    Unrelated,
}

fn classify_selected_control(
    route: CurrentPhysicalRecordPlacement,
    admitted: &[PersistedRecordIdentity],
) -> SelectedControlRole {
    let released_record = admitted.binary_search(&route.record()).is_ok();
    match route.content_class() {
        SelectedRecordContentClass::Blob(
            BlobRecordKind::ReclaimDescriptorV3
            | BlobRecordKind::OriginalDropReserved
            | BlobRecordKind::DropSetManifestV3,
        ) if released_record => SelectedControlRole::Released,
        SelectedRecordContentClass::Blob(
            BlobRecordKind::SessionDeclared
            | BlobRecordKind::SessionAbandoned
            | BlobRecordKind::GenerationPublished,
        ) => SelectedControlRole::Ordinary,
        SelectedRecordContentClass::Blob(
            BlobRecordKind::DropSetManifestV2
            | BlobRecordKind::ReclaimDescriptor
            | BlobRecordKind::OriginalDropReserved,
        ) => SelectedControlRole::FailedIngest,
        SelectedRecordContentClass::Blob(
            BlobRecordKind::DropSetManifest
            | BlobRecordKind::ReclaimDescriptorV2
            | BlobRecordKind::DropSetManifestV3
            | BlobRecordKind::ReclaimDescriptorV3,
        )
        | SelectedRecordContentClass::UnknownLegacy => SelectedControlRole::Unsupported,
        _ => SelectedControlRole::Unrelated,
    }
}

fn verify_ordinary_ingest_control(
    discovery: &mut worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery,
    route: CurrentPhysicalRecordPlacement,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    slices: &mut Vec<SelectedArtifactSlice>,
    storage: &mut HistoricalWalkStorage<'_, '_>,
) -> Result<(), Denial> {
    const MAX_ORDINARY_INGEST_CONTROL_BYTES: u64 = 4096;
    let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        return Err(Denial::UnsupportedSelectedPlacement);
    };
    if extent.payload_bytes() > MAX_ORDINARY_INGEST_CONTROL_BYTES {
        return Err(Denial::ControlFrame);
    }
    let (bytes, _) = read_extent_with_storage(discovery, format, extent, slices, storage)?;
    let store = discovery.store_identity().bytes();
    let admitted = match (route.content_class(), decode_blob_record(&bytes)) {
        (
            SelectedRecordContentClass::Blob(BlobRecordKind::SessionDeclared),
            Ok(BlobRecordV1::SessionDeclared(value)),
        ) => value.store() == store,
        (
            SelectedRecordContentClass::Blob(BlobRecordKind::SessionAbandoned),
            Ok(BlobRecordV1::SessionAbandoned(value)),
        ) => value.store() == store,
        (
            SelectedRecordContentClass::Blob(BlobRecordKind::GenerationPublished),
            Ok(BlobRecordV1::GenerationPublished(value)),
        ) => value.store() == store,
        _ => false,
    };
    storage.discard_vec(bytes)?;
    admitted.then_some(()).ok_or(Denial::ControlFrame)
}
