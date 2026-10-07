//! Fundable selected-frame walk around the shared failed-ingest predicates.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedRecordContentClass,
};

use super::super::super::control_frames::SelectedArtifactSlice;
use super::super::super::SelectedMediaRejoinDenial as Denial;
use super::semantic::{descriptor_matches_manifest, source_matches, verify_reservation};
use super::{FailedIngestFrameStorage, FailedIngestRoutes};

#[path = "core/observation.rs"]
mod observation;
use observation::ObservedFailedIngestControls;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct ProvenDrop {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) record:
        PersistedRecordIdentity,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) frame_digest: [u8; 32],
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) candidate_generation:
        u64,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_with_storage<
    S: FailedIngestFrameStorage,
    R: FailedIngestRoutes + ?Sized,
>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &R,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
    storage: &mut S,
) -> Result<Vec<ProvenDrop>, Denial> {
    let mut observed = ObservedFailedIngestControls::prepare(routes, storage)?;
    for route in routes.ordered() {
        observed.observe_one(discovery, route, format, slices, max_slices, storage)?;
    }
    let store = discovery.store_identity().bytes();
    verify_sources(
        discovery,
        routes,
        format,
        selected_generation,
        checkpoint_sequence,
        store,
        &observed,
        slices,
        max_slices,
        storage,
    )?;
    let proven = join_controls(&observed, store, selected_generation, storage)?;
    observed.discard(storage)?;
    Ok(proven)
}

#[allow(clippy::too_many_arguments)]
fn verify_sources<S: FailedIngestFrameStorage, R: FailedIngestRoutes + ?Sized>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &R,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    store: [u8; 16],
    observed: &ObservedFailedIngestControls,
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
    storage: &mut S,
) -> Result<(), Denial> {
    for (_, manifest) in &observed.manifests {
        if manifest.store != store
            || manifest
                .selected_slot
                .is_some_and(|slot| slot > selected_generation)
        {
            return Err(Denial::CertificateRoster);
        }
        verify_source(
            discovery,
            routes,
            format,
            manifest.basis,
            store,
            checkpoint_sequence,
            &observed.publications,
            slices,
            max_slices,
            storage,
        )?;
    }
    Ok(())
}

fn join_controls<S: FailedIngestFrameStorage>(
    observed: &ObservedFailedIngestControls,
    store: [u8; 16],
    selected_generation: u64,
    storage: &mut S,
) -> Result<Vec<ProvenDrop>, Denial> {
    let mut proven = storage.reserve_vec::<ProvenDrop>(observed.descriptors.len())?;
    for descriptor in &observed.descriptors {
        let manifest = observed
            .manifests
            .binary_search_by_key(&descriptor.manifest, |(record, _)| *record)
            .ok()
            .map(|index| &observed.manifests[index].1)
            .ok_or(Denial::CertificateRoster)?;
        if !descriptor_matches_manifest(descriptor, manifest, store, selected_generation) {
            return Err(Denial::CertificateRoster);
        }
        proven.push(ProvenDrop {
            record: descriptor.record,
            frame_digest: descriptor.frame_digest,
            candidate_generation: descriptor.candidate_generation,
        });
    }
    for reservation in &observed.reservations {
        verify_reservation(
            &observed.manifests,
            *reservation,
            store,
            selected_generation,
        )?;
    }
    Ok(proven)
}

#[allow(clippy::too_many_arguments)]
fn verify_source<S: FailedIngestFrameStorage, R: FailedIngestRoutes + ?Sized>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &R,
    format: PhysicalRecordFormatDeclaration,
    basis: worth_store_physical_format::FailedIngestReclaimBasisV1,
    store: [u8; 16],
    checkpoint_sequence: u64,
    publications: &[worth_store_physical_format::BlobGenerationPublicationV1],
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
    storage: &mut S,
) -> Result<(), Denial> {
    let declaration_route = route_for(routes, basis.declaration_record())?;
    let abandoned_route = route_for(routes, basis.abandoned_record())?;
    if declaration_route.content_class()
        != SelectedRecordContentClass::Blob(BlobRecordKind::SessionDeclared)
        || abandoned_route.content_class()
            != SelectedRecordContentClass::Blob(BlobRecordKind::SessionAbandoned)
    {
        return Err(Denial::CertificateRoster);
    }
    let declaration_bytes = storage.read_selected(
        discovery,
        format,
        declaration_route,
        4096,
        slices,
        max_slices,
    )?;
    let abandoned_bytes =
        storage.read_selected(discovery, format, abandoned_route, 4096, slices, max_slices)?;
    let BlobRecordV1::SessionDeclared(declaration) =
        decode_blob_record(&declaration_bytes).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let BlobRecordV1::SessionAbandoned(abandoned) =
        decode_blob_record(&abandoned_bytes).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let valid = source_matches(
        basis,
        declaration,
        &declaration_bytes,
        abandoned,
        &abandoned_bytes,
        store,
        checkpoint_sequence,
        publications,
    );
    storage.discard_vec(declaration_bytes)?;
    storage.discard_vec(abandoned_bytes)?;
    valid.then_some(()).ok_or(Denial::CertificateRoster)
}

fn route_for<R: FailedIngestRoutes + ?Sized>(
    routes: &R,
    record: PersistedRecordIdentity,
) -> Result<CurrentPhysicalRecordPlacement, Denial> {
    routes.route(record).ok_or(Denial::CertificateRoster)
}
