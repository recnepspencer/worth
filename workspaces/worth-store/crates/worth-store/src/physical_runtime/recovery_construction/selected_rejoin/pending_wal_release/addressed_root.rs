//! One addressed root/free pair is read from Store-qualified media and tied
//! to the complete semantic transcript carried by a checked C.8 chain edge.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};

use super::super::{tier, SelectedMediaRejoinDenial as Denial};

pub(super) struct AddressedRoot {
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) free: DurableFreeSpaceManifestHeader,
}

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    generation: u64,
    node_capacity: u16,
    format: PhysicalRecordFormatDeclaration,
    expected: PhysicalInventoryTranscriptV1,
    raw_root_sha256: Option<[u8; 32]>,
) -> Result<AddressedRoot, Denial> {
    let maximum = u64::from(format.page_size().bytes());
    let root_bytes = discovery
        .read_root_manifest(generation, maximum)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingRoot)?;
    let (root, observed_format) = DurablePhysicalRootManifest::decode(&root_bytes, node_capacity)
        .map_err(|_| Denial::RootBinding)?;
    let actual_root_sha256: [u8; 32] = Sha256::digest(&root_bytes).into();
    if observed_format != format
        || root.generation() != generation
        || root.encode(format) != root_bytes
        || raw_root_sha256.is_some_and(|digest| digest != actual_root_sha256)
    {
        return Err(Denial::RootBinding);
    }
    let free_bytes = discovery
        .read_free_space_manifest(generation, maximum)
        .map_err(Denial::Discovery)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let free = tier::selection::validate_selected_free_header(
        &free_bytes,
        discovery.store_identity(),
        format,
        &root,
    )?;
    if free.encode(format) != free_bytes || !expected.matches_headers(&root, &free, format) {
        return Err(Denial::RootBinding);
    }
    Ok(AddressedRoot { root, free })
}
