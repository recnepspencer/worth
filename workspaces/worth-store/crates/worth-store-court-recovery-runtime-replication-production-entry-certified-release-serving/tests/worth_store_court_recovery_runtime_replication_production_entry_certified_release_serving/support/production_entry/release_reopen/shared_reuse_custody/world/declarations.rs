//! Store-issued publications and actual shared extent placement for the journey.

pub(crate) use std::num::NonZeroU64;

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PublishedBlobGeneration,
};

pub(crate) use crate::production_entry::release_reopen::shared_reuse_custody::CHUNK;

pub(crate) fn release_operation_budget() -> NonZeroU64 {
    // Preserve the KiB64 world's 32 MiB mutation envelope and fund the two
    // maximum framed checkpoint commands added by residency phase 1 separately.
    // These release/reopen journeys make no narrow-memory admission claim.
    let command = worth_store_physical_format::MAX_CHECKPOINT_BINDING_RECORD_BYTES
        + worth_store_physical_format::CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES;
    std::num::NonZeroU64::new((32 << 20) + 2 * command as u64).unwrap()
}

pub(crate) fn shared_placement(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedRecordPlacementPolicy {
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(512).unwrap())
        .admit(format)
        .unwrap()
}

pub(crate) fn payload() -> Vec<u8> {
    let mut bytes = vec![0x5a; CHUNK + 17];
    bytes[CHUNK..].fill(0x7c);
    bytes
}

pub(crate) fn publish_one(
    world: &PhysicalResidencyStoreWorld,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = world.serving().blobs().unwrap();
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let object = blobs
        .issue_object_id(limits)
        .expect("Store-issued object identity");
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), (CHUNK / 2) as u64, limits)
        .expect("genuine shared placement ingest");
    for piece in payload.chunks(CHUNK / 2) {
        ingest.push(piece).expect("genuine shared chunk");
    }
    ingest
        .finish()
        .expect("shared placement publication and index both selected")
}
