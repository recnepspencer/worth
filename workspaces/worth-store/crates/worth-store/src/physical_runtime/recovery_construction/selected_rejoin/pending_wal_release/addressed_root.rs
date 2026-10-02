//! One addressed root/free pair is read from Store-qualified media and tied
//! to the complete semantic transcript carried by a checked C.8 chain edge.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use super::super::tier::routes::RouteWalkStorage;
use super::super::{
    control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint},
    tier, SelectedMediaRejoinDenial as Denial,
};

pub(super) struct AddressedRoot {
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) free: DurableFreeSpaceManifestHeader,
    pub(super) free_sha256: [u8; 32],
    pub(super) fingerprint: SelectedControlMediaFingerprint,
}

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    generation: u64,
    node_capacity: u16,
    format: PhysicalRecordFormatDeclaration,
    expected: PhysicalInventoryTranscriptV1,
    raw_root_sha256: Option<[u8; 32]>,
) -> Result<AddressedRoot, Denial> {
    observe_inner(
        discovery,
        generation,
        node_capacity,
        format,
        expected,
        raw_root_sha256,
        &mut (),
        false,
    )
}

pub(super) fn observe_with_storage<S: RouteWalkStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    generation: u64,
    node_capacity: u16,
    format: PhysicalRecordFormatDeclaration,
    expected: PhysicalInventoryTranscriptV1,
    raw_root_sha256: Option<[u8; 32]>,
    storage: &mut S,
) -> Result<AddressedRoot, Denial> {
    observe_inner(
        discovery,
        generation,
        node_capacity,
        format,
        expected,
        raw_root_sha256,
        storage,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn observe_inner<S: RouteWalkStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    generation: u64,
    node_capacity: u16,
    format: PhysicalRecordFormatDeclaration,
    expected: PhysicalInventoryTranscriptV1,
    raw_root_sha256: Option<[u8; 32]>,
    storage: &mut S,
    record_headers: bool,
) -> Result<AddressedRoot, Denial> {
    let maximum = u64::from(format.page_size().bytes());
    let mut slices =
        storage.reserve_vec::<SelectedArtifactSlice>(usize::from(record_headers) * 2)?;
    let root_frame = storage.read_page(
        discovery,
        RecordArtifactFile::RootManifest { generation },
        maximum,
    )?;
    let root_bytes = root_frame.bytes().ok_or(Denial::MissingRoot)?;
    let (root, observed_format) = DurablePhysicalRootManifest::decode(root_bytes, node_capacity)
        .map_err(|_| Denial::RootBinding)?;
    let actual_root_sha256: [u8; 32] = Sha256::digest(root_bytes).into();
    let reserved = storage.reserve_vec::<u8>(root.encoded_frame_bytes())?;
    let encoded = root
        .encode_in_reserved(format, reserved)
        .ok_or(Denial::RootBinding)?;
    let canonical = encoded.as_slice() == root_bytes;
    storage.discard_vec(encoded)?;
    if observed_format != format
        || root.generation() != generation
        || !canonical
        || raw_root_sha256.is_some_and(|digest| digest != actual_root_sha256)
    {
        return Err(Denial::RootBinding);
    }
    if record_headers {
        slices.push(
            SelectedArtifactSlice::observed(
                RecordArtifactFile::RootManifest { generation },
                0,
                root_bytes,
                true,
            )
            .ok_or(Denial::BoundExceeded)?,
        );
    }
    let free_frame = storage.read_page(
        discovery,
        RecordArtifactFile::FreeSpaceManifest { generation },
        maximum,
    )?;
    let free_bytes = free_frame.bytes().ok_or(Denial::MissingFrame)?;
    let free_sha256 = Sha256::digest(free_bytes).into();
    let free = tier::selection::validate_selected_free_header(
        free_bytes,
        discovery.store_identity(),
        format,
        &root,
    )?;
    let reserved = storage.reserve_vec::<u8>(free.encoded_frame_bytes())?;
    let encoded = free
        .encode_in_reserved(format, reserved)
        .ok_or(Denial::RootBinding)?;
    let canonical = encoded.as_slice() == free_bytes;
    storage.discard_vec(encoded)?;
    if !canonical || !expected.matches_headers_bytes(&root, &free, root_bytes, free_bytes) {
        return Err(Denial::RootBinding);
    }
    if record_headers {
        slices.push(
            SelectedArtifactSlice::observed(
                RecordArtifactFile::FreeSpaceManifest { generation },
                0,
                free_bytes,
                true,
            )
            .ok_or(Denial::BoundExceeded)?,
        );
    }
    storage.discard_frame(free_frame)?;
    storage.discard_frame(root_frame)?;
    Ok(AddressedRoot {
        root,
        free,
        free_sha256,
        fingerprint: SelectedControlMediaFingerprint::observed(slices),
    })
}
