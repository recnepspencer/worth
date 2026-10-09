//! A captured checkpoint prefix and a later object's drop coexist without losing either custody.

use super::*;
use worth_store::physical_runtime::production::PhysicalCheckpointStep;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, BlobReclaimReceipt, PhysicalCheckpointHandle,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    PersistedRecordIdentity, PhysicalCheckpointSource, ReleaseCustodyHeadKeyV1,
    CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

#[test]
fn paused_checkpoint_commits_its_prefix_then_later_drop_tail_reopens() {
    std::thread::Builder::new()
        .name("release-checkpoint-overlap".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("overlapping checkpoint worker")
        .join()
        .expect("overlapping checkpoint worker did not panic");
}

fn run() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_recovery_scope(
        "release-checkpoint-overlap",
        AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ),
        4,
        NonZeroU64::new(32 << 20).unwrap(),
    )
    .expect("same bounded production geometry as the release fixture");
    let serving = world.serving();
    let policy = serving.residency_observation().admitted_policy();
    assert_eq!(policy.operation_bytes(), 32 << 20);
    assert_eq!(
        policy.scope_bytes(PhysicalOperationAllocationScope::Recovery),
        32 << 20
    );

    // Both genuine publications finish indexing before any captured checkpoint.
    // A's release proof names its immutable publication, not the latest B marker.
    let (a_proof, a_key, a_publication) = publish(&world, 0x31, 0x52, 0x71);
    let (b_proof, b_key, b_publication) = publish(&world, 0x64, 0x65, 0x75);
    assert_ne!(a_key, b_key);
    assert_ne!(a_publication, b_publication);
    let baseline = recovery_bytes(serving);
    let a = partial_drop(&world, a_proof);
    assert!(a.dropped_records().contains(&a_publication));
    let a_funded = recovery_bytes(serving);
    assert!(
        a_funded > baseline,
        "A drop retains actual prefunded Recovery backing"
    );

    // Reclaim's native retirement progression already selected a covering
    // checkpoint before returning A's receipt. Observe that actual first fold.
    let initial_a_bytes = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (a_batches, initial_a_accumulator) =
        release_reopen::selected_release_certificates_from_bytes(&initial_a_bytes);
    let [a_batch] = a_batches.as_slice() else {
        panic!(
            "A's covering checkpoint must select one A Batch, got {}",
            a_batches.len()
        );
    };
    assert_eq!(a_batch.cumulative_dropped(), 1);
    assert!(!a_batch.terminal());
    assert_eq!(initial_a_accumulator.base().cumulative_dropped(), 1);
    let initial_a_heads =
        release_reopen::selected_head_oracle::selected_heads(world.root(), initial_a_accumulator);
    let [initial_a_head] = initial_a_heads.as_slice() else {
        panic!(
            "A's covering checkpoint must select one head, got {}",
            initial_a_heads.len()
        );
    };
    assert_eq!(initial_a_head.key(), a_key);
    assert_eq!(initial_a_head.cumulative_dropped(), 1);
    assert!(!initial_a_head.terminal());
    assert_eq!(
        initial_a_head.descriptor_record(),
        a_batch.descriptor_record()
    );
    assert_eq!(
        initial_a_head.descriptor_frame_sha256(),
        a_batch.descriptor_frame_sha256()
    );

    let gate =
        serving.pause_physical_checkpoint_at(PhysicalCheckpointStep::CandidateSynchronization);
    let checkpoint = start(serving, 0x7e);
    let captured = checkpoint.source();
    assert!(
        gate.await_arrival(),
        "A candidate must synchronize before B's drop"
    );
    let paused_funded = recovery_bytes(serving);
    assert!(
        paused_funded >= a_funded,
        "captured A custody cannot surrender its funding"
    );

    // This seam has released WAL and registry locks but still owns A's snapshot.
    // B completes without reading the still-old selected certificate as its result.
    let b = partial_drop(&world, b_proof);
    assert!(b.dropped_records().contains(&b_publication));
    assert!(
        recovery_bytes(serving) > paused_funded,
        "B's successor storage must be funded while A's captured storage is still live",
    );
    assert!(
        serving
            .records()
            .unwrap()
            .protected_root()
            .root()
            .generation()
            .get()
            > captured.root().generation(),
        "B really published a successor root while A's checkpoint was paused",
    );
    gate.release();
    let outcome = checkpoint.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "captured A prefix must complete despite B's later root: {outcome:?}"
    );

    let bytes = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let selected_source = PhysicalCheckpointSource::decode_stream_header_record(
        &bytes[..CHECKPOINT_STREAM_HEADER_RECORD_BYTES],
    )
    .unwrap();
    assert_eq!(selected_source, captured);
    let (carried_batches, a_accumulator) =
        release_reopen::selected_release_certificates_from_bytes(&bytes);
    assert!(
        carried_batches.is_empty(),
        "captured carryforward must not replay A or include B, got {} Batches",
        carried_batches.len()
    );
    assert_eq!(a_accumulator.base().cumulative_dropped(), 1);
    assert_eq!(
        a_accumulator.base().tip(),
        initial_a_accumulator.base().tip()
    );
    assert_eq!(a_accumulator.base().prior_cumulative_dropped(), 1);
    assert_eq!(
        a_accumulator.base().prior_checkpoint_sequence(),
        initial_a_accumulator.base().checkpoint().sequence().get()
    );
    assert_eq!(
        a_accumulator.prior_head_count(),
        initial_a_accumulator.head_count()
    );
    assert_eq!(
        a_accumulator.prior_head_roster_digest(),
        initial_a_accumulator.head_roster_digest()
    );
    assert_eq!(
        a_accumulator.head_roster_digest(),
        initial_a_accumulator.head_roster_digest()
    );
    let a_heads = release_reopen::selected_head_oracle::selected_heads(world.root(), a_accumulator);
    let [a_head] = a_heads.as_slice() else {
        panic!("captured A source must have exactly one selected head");
    };
    assert_eq!(
        a_head, initial_a_head,
        "captured carryforward preserves A's entire keyed semantic head byte-exactly"
    );

    let outcome = start(serving, 0x7f).wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "B's retained tail must remain checkpointable: {outcome:?}"
    );
    let bytes = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (b_batches, b_accumulator) =
        release_reopen::selected_release_certificates_from_bytes(&bytes);
    let [b_batch] = b_batches.as_slice() else {
        panic!("successor checkpoint must fold B only, not replay A's prefix");
    };
    assert_ne!(b_batch.descriptor_record(), a_batch.descriptor_record());
    assert_eq!(b_batch.ordinal(), 0);
    assert_eq!(b_batch.cumulative_dropped(), 2);
    assert!(!b_batch.terminal());
    assert_eq!(b_accumulator.base().prior_cumulative_dropped(), 1);
    assert_eq!(b_accumulator.base().cumulative_dropped(), 2);
    assert_eq!(b_accumulator.prior_head_count(), 1);
    assert_eq!(
        b_accumulator.prior_head_roster_digest(),
        a_accumulator.head_roster_digest()
    );
    assert_eq!(
        b_accumulator.base().prior_checkpoint_sequence(),
        a_accumulator.base().checkpoint().sequence().get()
    );
    let heads = release_reopen::selected_head_oracle::selected_heads(world.root(), b_accumulator);
    assert_eq!(heads.len(), 2);
    assert!(
        heads.contains(a_head),
        "A's keyed head carries byte-exactly into B's checkpoint"
    );
    let b_head = heads
        .iter()
        .find(|head| head.key() == b_key)
        .expect("B's distinct selected keyed head");
    assert_eq!(b_head.cumulative_dropped(), 1);
    assert!(!b_head.terminal());
    assert_eq!(b_head.descriptor_record(), b_batch.descriptor_record());
    assert_eq!(
        b_head.descriptor_frame_sha256(),
        b_batch.descriptor_frame_sha256()
    );

    let retained_root = world.retained_root();
    let root = retained_root.path().to_path_buf();
    drop(world);
    // C8 selects media afresh, Store independently rejoins it, and the one-shot
    // seal constructs Serving and another checkpoint. No live ledger crosses.
    super::super::recover(root, super::super::Mutation::None);
}

fn publish(
    world: &PhysicalResidencyStoreWorld,
    first_byte: u8,
    second_byte: u8,
    issuer_evidence: u8,
) -> (
    AdmittedBlobReleaseProof,
    ReleaseCustodyHeadKeyV1,
    PersistedRecordIdentity,
) {
    const CHUNK: usize = 64 << 10;
    let scope = admitted_blob_scope("c11.recovery.release.scope");
    let blobs = world.serving().blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let object = blobs
        .issue_object_id(limits)
        .expect("genuine object identity");
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
        .begin_ingest(declaration, world.placement(), CHUNK as u64, limits)
        .expect("genuine bounded ingest");
    ingest.push(&vec![first_byte; CHUNK]).unwrap();
    ingest.push(&vec![second_byte; CHUNK]).unwrap();
    let published = ingest
        .finish()
        .expect("publication must finish indexing; PublishedIndexPending is not fixture success");
    let generation = published.generation().sequence();
    drop(blobs);
    let marker = world
        .serving()
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("actual selected publication marker");
    let record = marker.record();
    let proof = AdmittedBlobReleaseProof::certification_admit(
        world.serving().store_identity().bytes(),
        object.bytes(),
        generation,
        record.allocation_epoch(),
        record.ordinal(),
        marker.encoded_digest(),
        [issuer_evidence; 32],
    )
    .unwrap();
    let key = ReleaseCustodyHeadKeyV1::new(object.bytes(), generation).unwrap();
    (proof, key, record)
}

fn partial_drop(
    world: &PhysicalResidencyStoreWorld,
    proof: AdmittedBlobReleaseProof,
) -> BlobReclaimReceipt {
    let request = BlobReclaimRequest::released(
        proof,
        world.placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(1024).unwrap(),
            NonZeroU64::new(64 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    let receipt = world
        .serving()
        .blobs()
        .unwrap()
        .reclaim(request)
        .expect("genuine release admission")
        .wait()
        .expect("partial drop completes");
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
    assert!(receipt.remaining_payload_records() > 0);
    receipt
}

fn start(serving: &ServingPhysicalRuntime, key: u8) -> PhysicalCheckpointHandle {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Failed(failure) => panic!("overlapping checkpoint failed: {failure:?}"),
        _ => panic!("overlapping checkpoint must admit"),
    }
}

fn recovery_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(PhysicalOperationAllocationScope::Recovery)
}
