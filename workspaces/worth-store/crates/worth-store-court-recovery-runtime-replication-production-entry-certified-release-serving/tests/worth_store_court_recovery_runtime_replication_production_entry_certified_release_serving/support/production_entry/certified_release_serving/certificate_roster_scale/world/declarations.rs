//! One genuine publication, then the maximum checkpoint-admissible series of
//! bounded partial V3 drops. This grows selected Batch custody without a
//! many-object identity scan or an artificial control roster.

pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, crate::production_entry::certified_release_serving::certificate_roster_scale::world};

pub(crate) use worth_proof::AdmittedBlobReleaseProof;
pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use worth_store_physical_format::MAX_CHECKPOINT_CERTIFICATE_RECORDS;
pub(crate) use {
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::PhysicalRetirementDenial,
};

// The production ingest declaration admits chunks from 64 KiB upward.
pub(crate) const CHUNK_BYTES: usize = 64 << 10;
pub(crate) const RELEASES: usize = MAX_CHECKPOINT_CERTIFICATE_RECORDS as usize - 1;
pub(crate) const CHUNKS: usize = RELEASES + 1;
// BlobTreeBuilder's 4,096-entry fanout writes one leaf/root for 64 chunks.
pub(crate) const TREE_NODE_RECORDS: usize = 1;
// An ingest checkpoints one resume frontier at its 64th chunk. The release
// owes it like the chunks and takes it before any of them.
pub(crate) const FRONTIER_RECORDS: usize = 1;
// The selected scan includes each payload, one publication, and at most four
// control records per completed V3 batch; 16 covers fixed ingress metadata.
pub(crate) const SELECTED_RECORD_LIMIT: u64 = (CHUNKS + 4 * RELEASES + 16) as u64;

pub(crate) fn checkpointed_partial_release_world() -> PhysicalResidencyStoreWorld {
    let started = std::time::Instant::now();
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_manifest_capacity(
        "recovered-ledger-partial-release",
        4,
    )
    .expect("initialize genuine compact-routing release world");
    let scope = admitted_blob_scope("c11.recovery.ledger.partial-release");
    let blobs = world.serving().blobs().expect("blob owner");
    let read = worth_store::physical_runtime::BlobReadLimits::new(
        std::num::NonZeroU64::new(SELECTED_RECORD_LIMIT).unwrap(),
    );
    let object = blobs.issue_object_id(read).expect("one object identity");
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        (CHUNKS * CHUNK_BYTES) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK_BYTES as u64, read)
        .expect("begin genuine multi-chunk ingest");
    let mut payload = [0_u8; CHUNK_BYTES];
    for chunk in 0..CHUNKS {
        payload[0] = u8::try_from(chunk).expect("bounded chunk ordinal");
        ingest
            .push(&payload)
            .expect("genuine distinct payload chunk");
    }
    let published = match ingest.finish() {
        Ok(published)
        | Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending {
            published,
            ..
        }) => published,
        Err(failure) => panic!("multi-chunk publication failed: {failure:?}"),
    };
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("selected publication for partially released object");
    assert_eq!(published.object(), object);
    assert_eq!(published.generation().sequence(), 1);
    let publication_record = marker.record();

    eprintln!(
        "roster scale: {CHUNKS} chunks ingested in {:?}",
        started.elapsed()
    );
    let checkpoint_path = world.root().join("families/checkpoint.current");
    assert_eq!(
        std::fs::read(&checkpoint_path).unwrap_err().kind(),
        std::io::ErrorKind::NotFound,
        "the initialized world has not checkpointed yet"
    );
    // Native retirement normally checkpoints each drop. Hold its owner before
    // any intent so all genuine pending drops reach one full-roster capture.
    let retirement = world
        .serving()
        .certification_try_pause_retirement()
        .expect("the quiescent runtime grants its retirement lane");
    for ordinal in 0..RELEASES {
        let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
            world.serving().store_identity().bytes(),
            object.bytes(),
            published.generation().sequence(),
            publication_record.allocation_epoch(),
            publication_record.ordinal(),
            marker.encoded_digest(),
            [0x71; 32],
        )
        .expect("owner-issued selected publication proof");
        let request = worth_store::physical_runtime::BlobReclaimRequest::released(
            proof,
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            worth_store::physical_runtime::BlobReclaimLimits::new(
                std::num::NonZeroU64::new(SELECTED_RECORD_LIMIT).unwrap(),
                std::num::NonZeroU64::new(32 << 20).unwrap(),
                std::num::NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        );
        let receipt = world
            .serving()
            .blobs()
            .unwrap()
            .reclaim(request)
            .unwrap_or_else(|failure| panic!("partial V3 admission {ordinal}: {failure:?}"))
            .wait()
            .unwrap_or_else(|failure| panic!("partial V3 completion {ordinal}: {failure:?}"));
        assert_eq!(
            receipt.disposition(),
            worth_store::physical_runtime::BlobReclaimDisposition::Dropped
        );
        assert_eq!(
            receipt.retirement(),
            worth_store::physical_runtime::BlobReclaimRetirement::Pending(
                worth_store::physical_runtime::PhysicalRetirementDenial::Waiting
            )
        );
        assert_eq!(
            std::fs::read(&checkpoint_path).unwrap_err().kind(),
            std::io::ErrorKind::NotFound,
            "partial V3 {ordinal} must remain pending for the first checkpoint"
        );
        assert_eq!(receipt.dropped_records().len(), 1, "partial V3 {ordinal}");
        if ordinal == 0 {
            assert_eq!(
                receipt.dropped_records(),
                &[publication_record],
                "the first post-order drop retires the selected publication"
            );
        } else {
            assert_ne!(receipt.dropped_records(), &[publication_record]);
        }
        // The publication is dropped first but is not counted among the
        // frontier, reachable chunks and tree nodes in
        // `remaining_payload_records`.
        assert_eq!(
            receipt.remaining_payload_records(),
            (CHUNKS + TREE_NODE_RECORDS + FRONTIER_RECORDS - ordinal) as u64,
            "partial V3 {ordinal} preserves the next live chunk"
        );
        if ordinal == 0 || (ordinal + 1) % 8 == 0 || ordinal + 1 == RELEASES {
            eprintln!(
                "roster scale: {}/{RELEASES} releases in {:?}",
                ordinal + 1,
                started.elapsed()
            );
        }
    }
    checkpoint(&world, 0xb7);
    drop(retirement);
    world
}

pub(crate) fn checkpoint(world: &PhysicalResidencyStoreWorld, key: u8) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(120_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("roster fixture checkpoint {key:#x} must admit")
    };
    assert!(
        matches!(
            handle.wait(),
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "roster fixture checkpoint {key:#x} must complete"
    );
}
