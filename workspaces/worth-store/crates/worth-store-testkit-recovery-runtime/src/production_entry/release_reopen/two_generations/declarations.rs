//! Two different published blob generations must not be forced into one V3
//! predecessor chain by the Store-wide release accumulator.

pub use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, std::num::NonZeroU16, std::num::NonZeroU64, worth_store_physical_format::PersistedRecordIdentity, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_store_physical_format::ReleaseCheckpointAccumulatorV2, worth_proof::TransitionOutcome, crate::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::release_reopen::released_world, crate::release_reopen::selected_head_oracle, crate::production_entry::release_reopen::selected_release_certificates, crate::production_entry::release_reopen::two_batch};

pub fn run(carryforward: bool) {
    let (world, accumulator) = world();
    if carryforward {
        let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
            worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x76; 32]),
            worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
        );
        let worth_proof::TransitionOutcome::Success(handle) =
            world.serving().checkpoints().start(checkpoint).into_raw()
        else {
            panic!("second generation carryforward checkpoint must admit")
        };
        assert!(matches!(
            handle.wait(),
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ));
        let (carried_batches, carried) = selected_release_certificates(&world);
        assert!(carried_batches.is_empty());
        assert_eq!(carried.base().tip(), accumulator.base().tip());
        assert_eq!(carried.prior_head_count(), accumulator.head_count());
        assert_eq!(
            carried.prior_head_roster_digest(),
            accumulator.head_roster_digest()
        );
    }
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    crate::production_entry::release_reopen::two_batch::recover(root);
}

pub fn world() -> (PhysicalResidencyStoreWorld, ReleaseCheckpointAccumulatorV2) {
    let (world, first, first_publication) = released_world(1024);
    assert_eq!(first.remaining_payload_records(), 0);
    let (_, first_accumulator) = selected_release_certificates(&world);
    assert!(first_accumulator.base().cumulative_dropped() > 0);
    let (accumulator, _) = release_second_object(&world, first_publication, first_accumulator);
    (world, accumulator)
}

pub fn release_second_object(
    world: &PhysicalResidencyStoreWorld,
    first_publication: PersistedRecordIdentity,
    first_accumulator: ReleaseCheckpointAccumulatorV2,
) -> (
    ReleaseCheckpointAccumulatorV2,
    worth_store_physical_format::ReleaseCustodyHeadKeyV1,
) {
    const CHUNK: usize = 64 << 10;
    let first_heads =
        crate::release_reopen::selected_head_oracle::selected_heads(world.retained_root().path(), first_accumulator);
    assert_eq!(first_heads.len(), 1);
    let scope = admitted_blob_scope("c11.recovery.release.second-generation");
    let blobs = world.serving().blobs().expect("blob owner");
    let read_limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let object = blobs
        .issue_object_id(read_limits)
        .expect("second object identity");
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, world.placement(), CHUNK as u64, read_limits)
        .expect("second generation ingest");
    ingest
        .push(&vec![0x64; CHUNK])
        .expect("second generation chunk");
    ingest
        .push(&vec![0x65; CHUNK])
        .expect("second generation trailing chunk");
    let published = match ingest.finish() {
        Ok(published) | Err(worth_store::physical_runtime::BlobIngestFailure::PublishedIndexPending { published, .. }) => {
            published
        }
        Err(failure) => panic!("second publication failed: {failure:?}"),
    };
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("second selected publication");
    assert_ne!(marker.record(), first_publication);
    let record = marker.record();
    let proof = worth_proof::AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        published.generation().sequence(),
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [0x75; 32],
    )
    .unwrap();
    let request = worth_store::physical_runtime::BlobReclaimRequest::released(
        proof,
        world.placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(1024).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    let second = world
        .serving()
        .blobs()
        .unwrap()
        .reclaim(request)
        .expect("second published generation release")
        .wait()
        .expect("second generation release completed");
    assert_eq!(second.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert!(second.dropped_records().contains(&record));
    let (batches, accumulator) = selected_release_certificates(world);
    assert_eq!(batches.len(), 1);
    assert_ne!(accumulator.base().tip(), first_accumulator.base().tip());
    assert_eq!(
        accumulator.base().tip(),
        batches[0].tip_provenance().unwrap()
    );
    assert_eq!(
        accumulator.base().prior_cumulative_dropped(),
        first_accumulator.base().cumulative_dropped()
    );
    let heads = crate::release_reopen::selected_head_oracle::selected_heads(world.retained_root().path(), accumulator);
    assert_eq!(heads.len(), 2);
    assert_ne!(heads[0].key(), heads[1].key());
    assert!(heads.contains(&first_heads[0]));
    let second_key = worth_store_physical_format::ReleaseCustodyHeadKeyV1::new(
        object.bytes(),
        published.generation().sequence(),
    )
    .unwrap();
    let second_head = heads.iter().find(|head| head.key() == second_key).unwrap();
    assert_eq!(
        second_head.descriptor_record(),
        batches[0].descriptor_record()
    );
    assert_eq!(
        second_head.descriptor_frame_sha256(),
        batches[0].descriptor_frame_sha256()
    );
    (accumulator, second_key)
}