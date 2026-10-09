use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits,
    BlobReclaimDisposition, BlobReclaimFailure, BlobReclaimLimitDenial, BlobReclaimLimits,
    BlobReclaimReceipt, BlobReclaimRequest, BlobReclaimRetirement, BlobReclaimRetirementBudget,
    BlobResumeToken, BlobTerminalLimits, ManifestEntryCapacity, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalMutationDeadline, PhysicalRecordId, PhysicalRecordPlacementPolicy, RecordByteLimit,
    RecordCountLimit, RecordReadLimits, RecordScanOutcome, RecordScanRequest,
    ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use super::{
    blob_crash::{establish_recovery_frontier, recover_closed_store},
    fixture::{
        admitted_blob_scope, configuration, serving_from_initialization_with_placement_and_tail,
        serving_from_open_with_retained_wal_tail,
    },
};

const CHUNK_BYTES: usize = 64 * 1024;
const CHUNKS: usize = 1025;
const INSPECTION_BYTE_DENIAL: u64 = 8 * 1024 * 1024;
// Three selected-root passes × 1,024 × (64KiB + 140B frame) = 201,756,672B.
// 256MiB covers custody too; encoded inspection is distinct from <=8MiB RAM.
const FULL_INSPECTION_ALLOWANCE: u64 = 256 * 1024 * 1024;
// The second batch reserves all 1,024 chunk extents before its first WAL effect.
// The common 64MiB tail cannot cover those extents plus retained WAL/candidates;
// admit a finite 128MiB tail on every open without weakening capacity checks.
const BATCH_RETAINED_WAL_TAIL_BYTES: u64 = 128 * 1024 * 1024;
const SCOPE: &str = "c11.blob.reclaim.batch-boundary.scope";

fn batch_placement() -> AdmittedRecordPlacementPolicy {
    let (format, _, _) = configuration();
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        // 8MiB index admits 2,047 ranges, enough for 1,024 isolated releases
        // plus frontier residue without future allocation coalescence.
        .arena_index_bytes(RecordByteLimit::new(8 * 1024 * 1024).unwrap())
        .admit(format)
        .unwrap()
}

#[test]
fn reclaim_drop_set_limit_rejects_1025_ids() {
    assert_eq!(
        BlobReclaimLimits::new(
            NonZeroU64::new(4096).unwrap(),
            NonZeroU64::new(FULL_INSPECTION_ALLOWANCE).unwrap(),
            NonZeroU16::new(1025).unwrap(),
        ),
        Err(BlobReclaimLimitDenial::DropSetTooLarge)
    );
}

/// Scheduled: 1,025 real C.5 appends and two multiscanned reclamations.
#[test]
#[ignore = "1,025 real 64 KiB chunk records plus bounded reclaim/recovery passes"]
fn abandoned_ingest_reclaims_exact_1024_id_batch_after_restart() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope(SCOPE);
    let placement = batch_placement();
    let serving = serving_from_initialization_with_placement_and_tail(
        directory.path(),
        placement,
        BATCH_RETAINED_WAL_TAIL_BYTES,
    );
    establish_recovery_frontier(&serving);
    let token = abandoned_1025_chunk_ingest(&serving, &scope, placement);
    serving.close();

    recover_closed_store(directory.path());
    let serving =
        serving_from_open_with_retained_wal_tail(directory.path(), BATCH_RETAINED_WAL_TAIL_BYTES);
    let selected = selected_chunk_identities(&serving, token);
    assert_eq!(selected.len(), CHUNKS, "all claims survived fresh recovery");
    let original_root = root_generation(&serving);
    let append_attempts = serving.media_counters().append_attempts();
    let denied =
        serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope, placement, INSPECTION_BYTE_DENIAL));
    assert!(matches!(
        denied,
        Err(BlobReclaimFailure::InspectedByteBoundExhausted)
    ));
    assert_eq!(root_generation(&serving), original_root);
    assert_eq!(serving.media_counters().append_attempts(), append_attempts);

    let mut first = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope, placement, FULL_INSPECTION_ALLOWANCE))
        .unwrap()
        .wait()
        .unwrap();
    finish_durable_batch(&serving, "frontier-and-last-chunk", &mut first);
    assert_durable_batch("frontier-and-last-chunk", &first);
    assert_eq!(first.remaining_payload_records(), 1024);
    assert_eq!(first.dropped_records().len(), 17);
    assert!(first.dropped_records().contains(&selected[1024]));
    assert_eq!(selected_chunk_identities(&serving, token), selected[..1024]);
    serving.close();

    recover_closed_store(directory.path());
    let serving =
        serving_from_open_with_retained_wal_tail(directory.path(), BATCH_RETAINED_WAL_TAIL_BYTES);
    assert_eq!(selected_chunk_identities(&serving, token), selected[..1024]);
    let mut second = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope, placement, FULL_INSPECTION_ALLOWANCE))
        .unwrap()
        .wait()
        .unwrap();
    finish_durable_batch(&serving, "1024-chunk-drop", &mut second);
    assert_durable_batch("1024-chunk-drop", &second);
    assert_eq!(second.dropped_records().len(), 1024);
    assert_eq!(second.remaining_payload_records(), 0);
    let mut expected_drop_ids = selected[..1024].to_vec();
    expected_drop_ids.sort_unstable();
    assert_eq!(second.dropped_records(), expected_drop_ids.as_slice());
    assert!(selected_chunk_identities(&serving, token).is_empty());
    serving.close();

    recover_closed_store(directory.path());
    let serving =
        serving_from_open_with_retained_wal_tail(directory.path(), BATCH_RETAINED_WAL_TAIL_BYTES);
    let root_before_repeat = root_generation(&serving);
    let repeated = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope, placement, FULL_INSPECTION_ALLOWANCE))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        repeated.disposition(),
        BlobReclaimDisposition::ProvenNoEffect
    );
    assert!(repeated.dropped_records().is_empty());
    assert_eq!(repeated.bytes_released(), 0);
    assert_eq!(root_generation(&serving), root_before_repeat);
    serving.close();
}

fn abandoned_1025_chunk_ingest(
    serving: &ServingPhysicalRuntime,
    scope: &worth_store::physical_runtime::AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
) -> BlobResumeToken {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(4096).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        (CHUNKS * CHUNK_BYTES) as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement, CHUNK_BYTES as u64, limits)
        .unwrap();
    let token = ingest.resume_token();
    let mut source = [0_u8; CHUNK_BYTES];
    for ordinal in 0..CHUNKS {
        source.fill((ordinal % 251) as u8);
        ingest.push(&source).unwrap();
        if (ordinal + 1) % 64 == 0 {
            physical_checkpoint(serving, ordinal + 1);
        }
    }
    assert_eq!(ingest.frontier().next_chunk_ordinal(), CHUNKS as u64);
    drop(ingest);
    blobs
        .abort_ingest(
            token,
            scope,
            placement,
            deadline(),
            BlobTerminalLimits::new(NonZeroU64::new(4096).unwrap()),
        )
        .unwrap();
    token
}

fn physical_checkpoint(serving: &ServingPhysicalRuntime, completed_chunks: usize) {
    let mut key = [0xb3; 32];
    key[..8].copy_from_slice(&(completed_chunks as u64).to_le_bytes());
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(120_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("checkpoint admission failed after {completed_chunks} chunks");
    };
    assert!(
        matches!(handle.wait(), PhysicalCheckpointOutcome::Completed(_)),
        "checkpoint did not complete after {completed_chunks} chunks"
    );
}

fn request<'a>(
    token: BlobResumeToken,
    scope: &'a worth_store::physical_runtime::AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
    inspected_bytes: u64,
) -> BlobReclaimRequest<'a> {
    BlobReclaimRequest::abandoned(
        token,
        scope,
        placement,
        deadline(),
        BlobReclaimLimits::new(
            NonZeroU64::new(4096).unwrap(),
            NonZeroU64::new(inspected_bytes).unwrap(),
            NonZeroU16::new(1024).unwrap(),
        )
        .unwrap(),
    )
}

fn assert_durable_batch(stage: &str, receipt: &BlobReclaimReceipt) {
    assert_eq!(
        receipt.disposition(),
        BlobReclaimDisposition::Dropped,
        "{stage}: {receipt:?}"
    );
    assert_eq!(
        receipt.retirement(),
        BlobReclaimRetirement::Completed,
        "{stage}: displaced={} released={} receipt={receipt:?}",
        receipt.displaced_extents().len(),
        receipt.bytes_released(),
    );
    assert!(receipt.bytes_released() > 0, "{stage}: {receipt:?}");
    assert_eq!(
        receipt.bytes_released(),
        receipt
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>()
    );
    let work = receipt.observation();
    assert!(work.inspected_payload_bytes() <= FULL_INSPECTION_ALLOWANCE);
    assert!(work.admitted_memory_bytes() <= INSPECTION_BYTE_DENIAL);
}

/// Native FIFO retirement may need more than the initial `wait()` quantum.
/// Neither a pending result nor exhausted continuation work proves release.
fn finish_durable_batch(
    serving: &ServingPhysicalRuntime,
    stage: &str,
    receipt: &mut BlobReclaimReceipt,
) {
    let expected = receipt
        .displaced_extents()
        .iter()
        .map(|extent| extent.range().length())
        .sum::<u64>();
    let budget = BlobReclaimRetirementBudget::new(
        NonZeroU32::new(128).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    let mut previous = receipt.bytes_released();
    assert!(previous <= expected, "{stage}: excessive native credit");
    for _ in 0..64 {
        if receipt.retirement() == BlobReclaimRetirement::Completed {
            assert_eq!(receipt.bytes_released(), expected, "{stage}: short credit");
            return;
        }
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(receipt, budget)
            .unwrap();
        let released = receipt.bytes_released();
        assert!(
            released >= previous && released <= expected,
            "{stage}: invalid native credit"
        );
        previous = released;
    }
    panic!("{stage}: retirement remained pending after bounded continuation: {receipt:?}");
}

/// Read one selected frame at a time; retain only the authenticated occurrence
/// identities needed to compare the two drop receipts with the source custody.
fn selected_chunk_identities(
    serving: &ServingPhysicalRuntime,
    token: BlobResumeToken,
) -> Vec<PersistedRecordIdentity> {
    let session: [u8; 16] = token.encode()[24..40].try_into().unwrap();
    let mut by_ordinal = vec![None; CHUNKS];
    let mut selected_chunks = 0;
    let mut declaration_seen = false;
    let mut abandonment_seen = false;
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(1).unwrap())
                .with_payload_limit(RecordByteLimit::new(236).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for row in batch.records() {
            let record = row.record_id();
            let bytes = read_selected(serving, record, row.declared_payload_bytes());
            if !bytes.starts_with(b"WRC11BLB") {
                continue;
            }
            match decode_blob_record(&bytes).unwrap() {
                BlobRecordV1::SessionDeclared(value) if value.session() == session => {
                    assert!(!declaration_seen, "duplicate selected declaration");
                    declaration_seen = true;
                }
                BlobRecordV1::SessionAbandoned(value) if value.session() == session => {
                    assert!(!abandonment_seen, "duplicate selected abandonment");
                    abandonment_seen = true;
                }
                BlobRecordV1::Chunk(value) if value.occurrence().session() == session => {
                    let ordinal = usize::try_from(value.occurrence().ordinal()).unwrap();
                    assert!(ordinal < CHUNKS);
                    assert_eq!(value.bytes().len(), CHUNK_BYTES);
                    assert!(value
                        .bytes()
                        .iter()
                        .all(|byte| *byte == (ordinal % 251) as u8));
                    let identity = persisted(record);
                    assert!(by_ordinal[ordinal].replace(identity).is_none());
                    selected_chunks += 1;
                }
                BlobRecordV1::GenerationPublished(value) if value.session() == session => {
                    panic!("abandoned ingest published generation")
                }
                _ => {}
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    assert!(
        declaration_seen && abandonment_seen,
        "selected terminal custody"
    );
    assert_eq!(selected_chunks, by_ordinal.iter().flatten().count());
    by_ordinal.into_iter().flatten().collect()
}

fn read_selected(
    serving: &ServingPhysicalRuntime,
    record: PhysicalRecordId,
    length: u64,
) -> Vec<u8> {
    let limit = RecordByteLimit::new(u32::try_from(length).unwrap()).unwrap();
    let mut stream = serving
        .records()
        .unwrap()
        .open(record, RecordReadLimits::new(limit))
        .unwrap();
    let mut bytes = vec![0; length as usize];
    let mut used = 0;
    while used < bytes.len() {
        let amount = stream.read_next(&mut bytes[used..]).unwrap();
        assert!(amount > 0, "selected frame truncated");
        used += amount;
    }
    bytes
}

fn persisted(record: PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}

fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

fn deadline() -> PhysicalMutationDeadline {
    PhysicalMutationDeadline::after_milliseconds(1_800_000).unwrap()
}
