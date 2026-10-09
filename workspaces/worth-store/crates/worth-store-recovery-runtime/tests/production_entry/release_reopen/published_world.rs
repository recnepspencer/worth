//! One genuine selected publication and owner-issued release proof, before reclaim.

use super::*;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalRecordFormatDeclaration,
};

pub(crate) fn create(
    activate_tier: bool,
    wal_segment_bytes: Option<NonZeroU64>,
    recovery_scope_bytes: Option<NonZeroU64>,
) -> (
    PhysicalResidencyStoreWorld,
    AdmittedBlobReleaseProof,
    PersistedRecordIdentity,
) {
    assert!(wal_segment_bytes.is_none() || recovery_scope_bytes.is_none());
    const CHUNK: usize = 64 << 10;
    let world = match (wal_segment_bytes, recovery_scope_bytes) {
        (Some(bytes), None) => {
            PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
                "release-checkpoint-reopen",
                bytes,
            )
        }
        (None, Some(bytes)) => {
            PhysicalResidencyStoreWorld::initialize_for_recovery_with_recovery_scope(
                "release-checkpoint-reopen",
                AdmittedPhysicalRecordFormat::admit(
                    PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
                ),
                4,
                bytes,
            )
        }
        (None, None) => {
            PhysicalResidencyStoreWorld::initialize_for_recovery("release-checkpoint-reopen")
        }
        (Some(_), Some(_)) => unreachable!("the fixture admits one policy override"),
    }
    .expect("initialize release world");
    if activate_tier {
        world
            .serving()
            .certification_activate_tier_epoch(world.placement())
            .expect("typed one-time tier activation before release");
    }
    let scope = admitted_blob_scope("c11.recovery.release.scope");
    let blobs = world.serving().blobs().expect("blob owner");
    let read_limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let object = blobs.issue_object_id(read_limits).expect("object identity");
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read_limits)
        .expect("begin ingest");
    ingest.push(&vec![0x31; CHUNK]).expect("first chunk");
    ingest.push(&vec![0x52; CHUNK]).expect("second chunk");
    let published = match ingest.finish() {
        Ok(published) | Err(BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("publication failed: {failure:?}"),
    };
    drop(blobs);
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication");
    let record = marker.record();
    let proof = AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [0x71; 32],
    )
    .unwrap();
    (world, proof, record)
}
