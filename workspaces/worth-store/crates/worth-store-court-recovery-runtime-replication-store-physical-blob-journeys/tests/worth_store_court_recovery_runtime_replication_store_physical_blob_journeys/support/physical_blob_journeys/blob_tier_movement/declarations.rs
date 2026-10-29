//! Feature-only proof of the Store's real arena-segregated SourceCopy path.
//! Public Hot/Cold admission stays sealed until C.8 accepts the epoch witness.

pub(crate) use {std::num::NonZeroU64, std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobMovementFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendDenial, worth_store::physical_runtime::RecordAppendError};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::arena_tier_at_epoch, worth_store_physical_format::BootstrapCatalog, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::PhysicalTierClass};

pub(crate) use {crate::physical_blob_journeys::blob_copy_observer::current_extent_route, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

pub(crate) fn assert_chunk(read: &mut worth_store::physical_runtime::BlobReadSession<'_>, expected: &[u8]) {
    let mut bytes = vec![0; expected.len()];
    assert_eq!(read.read_next(&mut bytes).unwrap(), expected.len());
    assert_eq!(bytes, expected);
}

pub(crate) fn selected_root_anchor(root: &Path) -> Option<[u8; 32]> {
    let catalog = worth_store_physical_format::BootstrapCatalog::decode(
        &std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    let generation = catalog.current_root().generation().get();
    let bytes = std::fs::read(root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    )))
    .unwrap();
    worth_store_physical_format::DurablePhysicalRootManifest::decode(&bytes, 64)
        .unwrap()
        .0
        .tier_epoch_anchor()
}

pub(crate) fn copy_checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ordinal: u8,
) -> worth_store::physical_runtime::CompletedPhysicalCheckpoint {
    use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest};
    let mut key = [219; 32];
    key[0] = ordinal;
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(30_000).unwrap()),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("tier copy checkpoint {ordinal} must admit");
    };
    let PhysicalCheckpointOutcome::Completed(completed) = handle.wait() else {
        panic!("tier copy checkpoint {ordinal} must complete");
    };
    completed
}