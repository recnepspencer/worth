#![cfg(feature = "certification-test-authority")]

use std::{fs, num::NonZeroU64, path::Path};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobReadFailure, BlobReadLimits,
    PhysicalMutationDeadline, PublishedBlobGeneration,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{BlobRecordDenial, DecodedBlobChunkFrameV1};

use super::{
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const C5_HEADER: usize = 48;
const C11_HEADER: usize = 48;
const C11_STORED_DIGEST: usize = C11_HEADER + 48;
const C11_CONTENT: usize = C11_HEADER + 80;

#[test]
fn damaged_inner_digest_field_localizes_without_inventing_an_observed_digest() {
    let (root, scope, published, limits) = one_chunk_store("c11.digest-field.corrupt");
    let before = observe_closed_store_named(root.path(), "c11-digest-field", "before");
    let identity = mutate_selected_chunk(root.path(), &before, Mutation::StoredDigestField);
    let after = observe_closed_store_named(root.path(), "c11-digest-field", "damaged");
    let row = selected_chunk(&after);
    assert_eq!(row["identity"], identity);
    assert_eq!(row["outcome"]["posture"], "damaged", "{after}");

    let serving = serving_from_open(root.path());
    let blobs = serving.blobs().unwrap();
    let mut read = blobs.read(published, &scope, 0, 1, limits).unwrap();
    let error = read.read_next(&mut [0; 1]).unwrap_err();
    assert!(
        matches!(
            error,
            BlobReadFailure::ChunkFrameCorruption { ordinal: 0, .. }
        ),
        "stored-digest field damage has no trustworthy observed content digest: {error:?}"
    );
    let target = read
        .damaged_chunk_scrub_target()
        .unwrap()
        .expect("selected inner frame failure retains exact chunk identity");
    assert_eq!(
        format!(
            "blob-record:{}",
            hex_record(target.scope().blob_record_identity().unwrap().0)
        ),
        identity
    );
}

#[test]
fn malformed_chunk_length_localizes_to_selected_chunk_record() {
    let (root, scope, published, limits) = one_chunk_store("c11.chunk-length.corrupt");
    let before = observe_closed_store_named(root.path(), "c11-chunk-length", "before");
    let identity = mutate_selected_chunk(root.path(), &before, Mutation::ContentLengthField);
    let after = observe_closed_store_named(root.path(), "c11-chunk-length", "damaged");
    let row = selected_chunk(&after);
    assert_eq!(row["identity"], identity);
    assert_eq!(row["outcome"]["posture"], "damaged", "{after}");

    let serving = serving_from_open(root.path());
    let blobs = serving.blobs().unwrap();
    let mut read = blobs.read(published, &scope, 0, 1, limits).unwrap();
    let error = read.read_next(&mut [0; 1]).unwrap_err();
    assert!(
        matches!(
            error,
            BlobReadFailure::ChunkFrameCorruption { ordinal: 0, .. }
        ),
        "structurally malformed selected frame must identify its chunk: {error:?}"
    );
    let target = read.damaged_chunk_scrub_target().unwrap().unwrap();
    assert_eq!(
        format!(
            "blob-record:{}",
            hex_record(target.scope().blob_record_identity().unwrap().0)
        ),
        identity
    );
}

#[test]
fn valid_chunk_with_resealed_changed_content_blames_parent_edge_not_chunk() {
    let (root, scope, published, limits) = one_chunk_store("c11.valid-chunk.wrong-edge");
    let before = observe_closed_store_named(root.path(), "c11-edge-mismatch", "before");
    let identity = mutate_selected_chunk(root.path(), &before, Mutation::ValidChangedContent);
    let after = observe_closed_store_named(root.path(), "c11-edge-mismatch", "changed");
    let chunk = selected_chunk(&after);
    assert_eq!(chunk["identity"], identity);
    assert_eq!(chunk["outcome"]["posture"], "intact", "{after}");
    assert!(
        after["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["family"] == "blob_tree_node" && row["outcome"]["posture"] == "damaged"),
        "the selected parent edge, not the valid chunk, is inconsistent: {after}"
    );

    let serving = serving_from_open(root.path());
    let blobs = serving.blobs().unwrap();
    let mut read = blobs.read(published, &scope, 0, 1, limits).unwrap();
    let error = read.read_next(&mut [0; 1]).unwrap_err();
    assert!(matches!(error, BlobReadFailure::TreeDamaged), "{error:?}");
    assert!(
        read.damaged_chunk_scrub_target().unwrap().is_none(),
        "a valid chunk must not acquire a false chunk-damage target"
    );
}

#[derive(Clone, Copy)]
enum Mutation {
    StoredDigestField,
    ContentLengthField,
    ValidChangedContent,
}

fn one_chunk_store(
    scenario: &str,
) -> (
    tempfile::TempDir,
    AdmittedBlobScope,
    PublishedBlobGeneration,
    BlobReadLimits,
) {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope(scenario);
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let published = {
        let serving = serving_from_initialization(root.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(64 << 10).unwrap(),
            2,
            &scope,
            BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), 1, limits)
            .unwrap();
        ingest.push(&[0x51]).unwrap();
        ingest.push(&[0x52]).unwrap();
        let published = ingest.finish().unwrap();
        drop(blobs);
        serving.close();
        published
    };
    (root, scope, published, limits)
}

fn selected_chunk(report: &serde_json::Value) -> &serde_json::Value {
    let rows = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == "blob_chunk_frame")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "one selected chunk: {report}");
    rows[0]
}

fn mutate_selected_chunk(root: &Path, report: &serde_json::Value, mutation: Mutation) -> String {
    let row = selected_chunk(report);
    assert_eq!(row["outcome"]["posture"], "intact", "{report}");
    let identity = row["identity"].as_str().unwrap().to_owned();
    let path = root.join(row["path"].as_str().unwrap());
    let mut media = fs::read(&path).unwrap();
    let chunks = (0..media.len().saturating_sub(C11_HEADER))
        .filter(|offset| media[*offset..].starts_with(b"WRC11BLB") && media[*offset + 8] == 2)
        .collect::<Vec<_>>();
    assert_eq!(chunks.len(), 1, "one physical selected chunk frame");
    let inner_start = chunks[0];
    let inner_len = C11_HEADER
        + u32::from_le_bytes(
            media[inner_start + 12..inner_start + 16]
                .try_into()
                .unwrap(),
        ) as usize;
    let outer = (inner_start.saturating_sub(1024)..inner_start)
        .filter(|offset| media[*offset..].starts_with(b"WRC5FRM\0"))
        .filter_map(|offset| {
            let length = C5_HEADER
                + u32::from_le_bytes(media[offset + 24..offset + 28].try_into().ok()?) as usize;
            (offset + length >= inner_start + inner_len && offset + length <= media.len())
                .then_some((offset, length))
        })
        .collect::<Vec<_>>();
    assert_eq!(outer.len(), 1, "one outer frame contains the small chunk");
    let (outer_start, outer_len) = outer[0];
    let inner = &mut media[inner_start..inner_start + inner_len];
    assert!(DecodedBlobChunkFrameV1::decode(inner).is_ok());
    match mutation {
        Mutation::StoredDigestField => inner[C11_STORED_DIGEST] ^= 0x01,
        Mutation::ContentLengthField => inner[C11_CONTENT + 8] ^= 0x01,
        Mutation::ValidChangedContent => {
            inner[C11_CONTENT + 12] ^= 0x01;
            let content_digest = Sha256::digest(&inner[C11_CONTENT..]);
            inner[C11_STORED_DIGEST..C11_STORED_DIGEST + 32].copy_from_slice(&content_digest);
        }
    }
    let mut envelope = Sha256::new();
    envelope.update(&inner[..16]);
    envelope.update(&inner[C11_HEADER..]);
    inner[16..C11_HEADER].copy_from_slice(&envelope.finalize());
    match mutation {
        Mutation::StoredDigestField => assert_eq!(
            DecodedBlobChunkFrameV1::decode(inner),
            Err(BlobRecordDenial::IntegrityMismatch)
        ),
        Mutation::ContentLengthField => assert_eq!(
            DecodedBlobChunkFrameV1::decode(inner),
            Err(BlobRecordDenial::InvalidChunkLength)
        ),
        Mutation::ValidChangedContent => assert!(DecodedBlobChunkFrameV1::decode(inner).is_ok()),
    }
    let outer = &mut media[outer_start..outer_start + outer_len];
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    fs::write(path, media).unwrap();
    identity
}

fn hex_record(record: worth_store_physical_format::PersistedRecordIdentity) -> String {
    let mut encoded = [0_u8; 24];
    encoded[..16].copy_from_slice(&record.allocation_epoch());
    encoded[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    encoded.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in prefix.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}
