pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path};

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PublishedBlobGeneration,
};
pub(crate) use {
    worth_store_physical_format::BlobRecordDenial,
    worth_store_physical_format::DecodedBlobChunkFrameV1,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) const C5_HEADER: usize = 48;
pub(crate) const C11_HEADER: usize = 48;
pub(crate) const C11_STORED_DIGEST: usize = C11_HEADER + 48;
pub(crate) const C11_CONTENT: usize = C11_HEADER + 80;

#[derive(Clone, Copy)]
pub(crate) enum Mutation {
    StoredDigestField,
    ContentLengthField,
    ValidChangedContent,
}

pub(crate) fn one_chunk_store(
    scenario: &str,
) -> (
    tempfile::TempDir,
    AdmittedBlobScope,
    PublishedBlobGeneration,
    BlobReadLimits,
) {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope(scenario);
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let published = {
        let serving = serving_from_initialization(root.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(64 << 10).unwrap(),
            2,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), 1, limits)
            .unwrap();
        ingest.push(&[0x51]).unwrap();
        ingest.push(&[0x52]).unwrap();
        let published = ingest.finish().unwrap();
        serving.close();
        published
    };
    (root, scope, published, limits)
}

pub(crate) fn selected_chunk(report: &serde_json::Value) -> &serde_json::Value {
    let rows = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == "blob_chunk_frame")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "one selected chunk: {report}");
    rows[0]
}

pub(crate) fn mutate_selected_chunk(
    root: &Path,
    report: &serde_json::Value,
    mutation: Mutation,
) -> String {
    let row = selected_chunk(report);
    assert_eq!(row["outcome"]["posture"], "intact", "{report}");
    let identity = row["identity"].as_str().unwrap().to_owned();
    let path = root.join(row["path"].as_str().unwrap());
    let mut media = std::fs::read(&path).unwrap();
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
    assert!(worth_store_physical_format::DecodedBlobChunkFrameV1::decode(inner).is_ok());
    match mutation {
        crate::physical_blob_journeys::blob_digest_corruption::Mutation::StoredDigestField => {
            inner[C11_STORED_DIGEST] ^= 0x01
        }
        crate::physical_blob_journeys::blob_digest_corruption::Mutation::ContentLengthField => {
            inner[C11_CONTENT + 8] ^= 0x01
        }
        crate::physical_blob_journeys::blob_digest_corruption::Mutation::ValidChangedContent => {
            inner[C11_CONTENT + 12] ^= 0x01;
            let content_digest = sha2::Sha256::digest(&inner[C11_CONTENT..]);
            inner[C11_STORED_DIGEST..C11_STORED_DIGEST + 32].copy_from_slice(&content_digest);
        }
    }
    let mut envelope = sha2::Sha256::new();
    envelope.update(&inner[..16]);
    envelope.update(&inner[C11_HEADER..]);
    inner[16..C11_HEADER].copy_from_slice(&envelope.finalize());
    match mutation {
        crate::physical_blob_journeys::blob_digest_corruption::Mutation::StoredDigestField => {
            assert_eq!(
                worth_store_physical_format::DecodedBlobChunkFrameV1::decode(inner),
                Err(worth_store_physical_format::BlobRecordDenial::IntegrityMismatch)
            )
        }
        crate::physical_blob_journeys::blob_digest_corruption::Mutation::ContentLengthField => {
            assert_eq!(
                worth_store_physical_format::DecodedBlobChunkFrameV1::decode(inner),
                Err(worth_store_physical_format::BlobRecordDenial::InvalidChunkLength)
            )
        }
        crate::physical_blob_journeys::blob_digest_corruption::Mutation::ValidChangedContent => {
            assert!(worth_store_physical_format::DecodedBlobChunkFrameV1::decode(inner).is_ok())
        }
    }
    let outer = &mut media[outer_start..outer_start + outer_len];
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    std::fs::write(path, media).unwrap();
    identity
}

pub(crate) fn hex_record(record: worth_store_physical_format::PersistedRecordIdentity) -> String {
    let mut encoded = [0_u8; 24];
    encoded[..16].copy_from_slice(&record.allocation_epoch());
    encoded[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    encoded.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
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
