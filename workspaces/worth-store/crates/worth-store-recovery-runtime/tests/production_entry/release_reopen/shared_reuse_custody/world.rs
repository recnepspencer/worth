//! Store-issued publications and actual shared extent placement for the journey.

use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    AdmittedBlobScope, AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy,
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, ManifestEntryCapacity,
    PhysicalMutationDeadline, PhysicalRecordPlacementPolicy, PublishedBlobGeneration,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::CHUNK;

pub(super) fn shared_placement(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(512).unwrap())
        .admit(format)
        .unwrap()
}

pub(super) fn payload() -> Vec<u8> {
    let mut bytes = vec![0x5a; CHUNK + 17];
    bytes[CHUNK..].fill(0x7c);
    bytes
}

pub(super) fn publish_one(
    world: &PhysicalResidencyStoreWorld,
    scope: &AdmittedBlobScope,
    payload: &[u8],
) -> PublishedBlobGeneration {
    let blobs = world.serving().blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let object = blobs
        .issue_object_id(limits)
        .expect("Store-issued object identity");
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        payload.len() as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
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
