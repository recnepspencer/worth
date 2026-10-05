use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, BlobResumeToken,
    PhysicalMutationDeadline, PhysicalRecordId, RecordByteLimit, RecordCountLimit,
    RecordReadLimits, RecordScanOutcome, RecordScanRequest, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, BlobSessionFrontierV1, PersistedRecordIdentity,
};

use super::fixture::{
    admitted_blob_scope, placement, serving_from_initialization, serving_from_open,
};

#[path = "blob_frontier/reused_last_chunk.rs"]
mod reused_last_chunk;

const CHUNK: usize = 64 * 1024;

#[test]
fn explicit_frontier_persists_only_completed_prefix_and_selected_custody() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.frontier.selected.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(32).unwrap());
    let serving = serving_from_initialization(directory.path());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 2) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, limits)
        .unwrap();
    let token = ingest.resume_token();
    assert_eq!(BlobResumeToken::decode(&token.encode()), Ok(token));

    let declared_root = selected_root_generation(&serving);
    let empty = ingest.checkpoint().unwrap();
    assert_eq!((empty.bytes(), empty.next_chunk_ordinal()), (0, 0));
    assert_eq!(selected_root_generation(&serving), declared_root);

    let full_chunk = (0..CHUNK)
        .map(|index| ((index * 17 + 3) % 251) as u8)
        .collect::<Vec<_>>();
    ingest.push(&full_chunk[..CHUNK / 2]).unwrap();
    ingest.push(&full_chunk[CHUNK / 2..]).unwrap();
    ingest.push(&[0xa5; 17]).unwrap();
    let before_frontier = selected_root_generation(&serving);
    assert_eq!(
        (
            ingest.frontier().bytes(),
            ingest.frontier().next_chunk_ordinal()
        ),
        (CHUNK as u64, 1),
        "the 17-byte partial chunk is not a durable prefix claim"
    );
    let checkpoint = ingest.checkpoint().unwrap();
    assert_eq!(
        (checkpoint.bytes(), checkpoint.next_chunk_ordinal()),
        (CHUNK as u64, 1)
    );
    let after_frontier = selected_root_generation(&serving);
    assert_eq!(after_frontier, before_frontier + 1);
    assert_eq!(ingest.checkpoint().unwrap(), checkpoint);
    assert_eq!(selected_root_generation(&serving), after_frontier);

    drop(ingest);
    drop(blobs);
    serving.close();

    let reopened = serving_from_open(directory.path());
    let selected = selected_blob_records(&reopened);
    assert_eq!(
        selected.len(),
        3,
        "declaration, one chunk, one frontier only"
    );
    let mut declaration = None;
    let mut chunk = None;
    let mut frontier = None;
    for (record, bytes) in selected {
        match decode_blob_record(&bytes).unwrap() {
            BlobRecordV1::SessionDeclared(value) => {
                let digest: [u8; 32] = Sha256::digest(&bytes).into();
                assert!(declaration.replace((record, digest, value)).is_none());
            }
            BlobRecordV1::Chunk(value) => {
                assert_eq!(value.bytes(), full_chunk.as_slice());
                assert_eq!(value.occurrence().ordinal(), 0);
                assert!(chunk.replace((record, value.stored_digest())).is_none());
            }
            BlobRecordV1::SessionFrontier(value) => {
                assert!(frontier.replace(value).is_none());
            }
            other => panic!("unexpected selected blob record: {other:?}"),
        }
    }
    let (declaration_record, declaration_digest, declaration) = declaration.unwrap();
    let (chunk_record, chunk_digest) = chunk.unwrap();
    let frontier: BlobSessionFrontierV1 = frontier.unwrap();
    assert_eq!(declaration.object(), object.bytes());
    assert_eq!(declaration.declared_bytes(), (CHUNK * 2) as u64);
    assert_eq!(frontier.store(), declaration.store());
    assert_eq!(frontier.session(), declaration.session());
    assert_record_binding(declaration_record, frontier.declaration_record());
    assert_eq!(frontier.declaration_digest(), declaration_digest);
    assert_record_binding(chunk_record, frontier.last_chunk_record());
    assert_eq!(frontier.last_chunk_digest(), chunk_digest);
    assert_eq!(
        (frontier.durable_bytes(), frontier.next_chunk_ordinal()),
        (CHUNK as u64, 1)
    );
    let token_bytes = token.encode();
    assert_eq!(&token_bytes[64..96], declaration_digest.as_slice());
    assert_eq!(&token_bytes[8..24], declaration.store().as_slice());
    assert_eq!(&token_bytes[24..40], declaration.session().as_slice());
    reopened.close();
}

#[test]
fn sixty_fourth_chunk_auto_publishes_one_frontier_not_one_per_chunk() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.frontier.periodic.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let serving = serving_from_initialization(directory.path());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (66 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(300_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, limits)
        .unwrap();
    let token = ingest.resume_token();
    let mut source = [0_u8; CHUNK / 2];
    for ordinal in 0..63_u8 {
        source.fill(ordinal);
        ingest.push(&source).unwrap();
        ingest.push(&source).unwrap();
    }
    assert_eq!(ingest.frontier().next_chunk_ordinal(), 63);
    assert_eq!(ingest.frontier().bytes(), (63 * CHUNK) as u64);
    let at_63 = selected_root_generation(&serving);
    let (controls_at_63, chunks_at_63) = selected_control_inventory(&serving);
    assert_eq!(controls_at_63.len(), 1, "only declaration before interval");
    assert_eq!(chunks_at_63.len(), 63);

    source.fill(63);
    ingest.push(&source).unwrap();
    ingest.push(&source).unwrap();
    let at_64 = selected_root_generation(&serving);
    assert_eq!(at_64, at_63 + 2, "one chunk and one automatic frontier");
    assert_eq!(ingest.frontier().next_chunk_ordinal(), 64);
    assert_eq!(ingest.frontier().bytes(), (64 * CHUNK) as u64);

    source.fill(64);
    ingest.push(&source).unwrap();
    ingest.push(&source).unwrap();
    assert_eq!(selected_root_generation(&serving), at_64 + 1);
    assert_eq!(ingest.frontier().next_chunk_ordinal(), 65);
    assert_eq!(ingest.frontier().bytes(), (65 * CHUNK) as u64);
    drop(ingest);
    drop(blobs);
    serving.close();

    let reopened = serving_from_open(directory.path());
    let (controls, chunks) = selected_control_inventory(&reopened);
    assert_eq!(chunks.len(), 65, "65 completed chunk records selected");
    assert_eq!(controls.len(), 2, "declaration plus one frontier only");
    let mut selected_declaration = None;
    let mut selected_frontier = None;
    for (record, bytes) in controls {
        match decode_blob_record(&bytes).unwrap() {
            BlobRecordV1::SessionDeclared(declaration) => {
                let digest: [u8; 32] = Sha256::digest(&bytes).into();
                assert!(selected_declaration
                    .replace((record, digest, declaration))
                    .is_none());
            }
            BlobRecordV1::SessionFrontier(frontier) => {
                assert!(selected_frontier.replace(frontier).is_none());
            }
            other => panic!("unexpected control record: {other:?}"),
        }
    }
    let (declaration_record, declaration_digest, declaration) = selected_declaration.unwrap();
    let frontier = selected_frontier.unwrap();
    assert_eq!(declaration.object(), object.bytes());
    assert_eq!(frontier.session(), declaration.session());
    assert_eq!(frontier.store(), declaration.store());
    assert_record_binding(declaration_record, frontier.declaration_record());
    assert_eq!(frontier.declaration_digest(), declaration_digest);
    assert_eq!(&token.encode()[64..96], declaration_digest.as_slice());
    assert_eq!(frontier.next_chunk_ordinal(), 64);
    assert_eq!(frontier.durable_bytes(), (64 * CHUNK) as u64);
    let (last_record, last_len) = chunks
        .into_iter()
        .find(|(record, _)| {
            record.allocation_epoch() == frontier.last_chunk_record().allocation_epoch()
                && record.ordinal() == frontier.last_chunk_record().ordinal()
        })
        .expect("frontier's last chunk must be selected");
    let last_bytes = read_selected_record(&reopened, last_record, last_len);
    let BlobRecordV1::Chunk(last_chunk) = decode_blob_record(&last_bytes).unwrap() else {
        panic!("frontier bound a non-chunk record");
    };
    assert_eq!(last_chunk.occurrence().ordinal(), 63);
    assert_eq!(last_chunk.bytes(), [63_u8; CHUNK].as_slice());
    assert_eq!(frontier.last_chunk_digest(), last_chunk.stored_digest());
    reopened.close();
}

fn selected_root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

pub(super) fn selected_blob_records(
    serving: &ServingPhysicalRuntime,
) -> Vec<(PhysicalRecordId, Vec<u8>)> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(236).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    let mut selected = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for row in batch.records() {
            selected.push((row.record_id(), row.declared_payload_bytes()));
        }
        if batch.is_complete() {
            break;
        }
    }
    drop(scan);
    selected
        .into_iter()
        .map(|(record, length)| (record, read_selected_record(serving, record, length)))
        .collect()
}

/// Scans selected routing while copying only the small declaration/frontier
/// controls. The 64 KiB chunk frames remain deferred metadata identities.
fn selected_control_inventory(
    serving: &ServingPhysicalRuntime,
) -> (
    Vec<(PhysicalRecordId, Vec<u8>)>,
    Vec<(PhysicalRecordId, u64)>,
) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(236).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    let mut controls = Vec::new();
    let mut deferred = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for (index, row) in batch.records().iter().enumerate() {
            match batch.payload(index) {
                Some(bytes) => controls.push((row.record_id(), bytes.to_vec())),
                None => deferred.push((row.record_id(), row.declared_payload_bytes())),
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    (controls, deferred)
}

fn read_selected_record(
    serving: &ServingPhysicalRuntime,
    record: PhysicalRecordId,
    length: u64,
) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut stream = reader
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(u32::try_from(length).unwrap()).unwrap()),
        )
        .unwrap();
    let mut bytes = Vec::with_capacity(length as usize);
    let mut frame = [0_u8; 8192];
    loop {
        let read = stream.read_next(&mut frame).unwrap();
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&frame[..read]);
    }
    assert_eq!(bytes.len() as u64, length);
    bytes
}

fn assert_record_binding(record: PhysicalRecordId, selected: PersistedRecordIdentity) {
    assert_eq!(record.allocation_epoch(), selected.allocation_epoch());
    assert_eq!(record.ordinal(), selected.ordinal());
}
