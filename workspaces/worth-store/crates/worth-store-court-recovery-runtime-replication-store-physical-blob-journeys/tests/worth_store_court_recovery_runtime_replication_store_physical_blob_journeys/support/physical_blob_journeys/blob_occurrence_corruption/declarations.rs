pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path};

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
};

pub(crate) const C5_HEADER: usize = 48;
pub(crate) const C5_EXTENT_METADATA: usize = 64;
pub(crate) const C11_HEADER: usize = 48;
pub(crate) const C11_CHUNK_CLAIM: usize = 80;

pub(crate) fn write_two_byte_blob(root: &Path) {
    let serving = serving_from_initialization(root);
    let blobs = serving.blobs().unwrap();
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(512).unwrap());
    let scope = admitted_blob_scope("c11.blob.occurrence.corruption.scope");
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(64 << 10).unwrap(),
        2,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(64).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
    )
    .unwrap();
    let mut session = blobs
        .begin_ingest(declaration, placement(), 1, limits)
        .unwrap();
    session.push(&[0x7e]).unwrap();
    session.push(&[0x7f]).unwrap();
    session.finish().unwrap();
    serving.close();
}

pub(crate) fn blob_rows<'a>(
    report: &'a serde_json::Value,
    family: &str,
) -> Vec<&'a serde_json::Value> {
    report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == family)
        .collect()
}

pub(crate) fn omit_occurrence_and_reseal_inner_and_outer_frames(root: &Path) {
    let arena_dir = root.join("families/records/arenas");
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(arena_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        for inner_offset in 0..bytes.len().saturating_sub(C11_HEADER) {
            let Some(outer_offset) = inner_offset.checked_sub(C5_HEADER + C5_EXTENT_METADATA)
            else {
                continue;
            };
            if bytes[inner_offset..].starts_with(b"WRC11BLB")
                && bytes[inner_offset + 8] == 2
                && bytes[outer_offset..].starts_with(b"WRC5FRM\0")
                && bytes[outer_offset + 8] == 4
            {
                let payload_len = u32::from_le_bytes(
                    bytes[outer_offset + 24..outer_offset + 28]
                        .try_into()
                        .unwrap(),
                ) as usize;
                let frame_len = C5_HEADER + payload_len;
                let inner_len = C11_HEADER
                    + u32::from_le_bytes(
                        bytes[inner_offset + 12..inner_offset + 16]
                            .try_into()
                            .unwrap(),
                    ) as usize;
                if outer_offset + frame_len <= bytes.len()
                    && inner_len + C5_EXTENT_METADATA == payload_len
                    && u32::from_le_bytes(
                        bytes[outer_offset + 104..outer_offset + 108]
                            .try_into()
                            .unwrap(),
                    ) as usize
                        == inner_len
                {
                    candidates.push((path.clone(), outer_offset, frame_len));
                }
            }
        }
    }
    assert_eq!(
        candidates.len(),
        1,
        "expected one production-written chunk frame"
    );
    let (path, outer_offset, frame_len) = candidates.pop().unwrap();
    let mut media = std::fs::read(&path).unwrap();
    let outer = &mut media[outer_offset..outer_offset + frame_len];
    let inner = &mut outer[C5_HEADER + C5_EXTENT_METADATA..];
    assert_eq!(inner[10..12], 1_u16.to_le_bytes());
    assert_eq!(inner.len(), C11_HEADER + C11_CHUNK_CLAIM + 12 + 2);
    let stored_content_digest = &inner[C11_HEADER + 48..C11_HEADER + 80];
    assert_eq!(
        stored_content_digest,
        sha2::Sha256::digest(&inner[C11_HEADER + C11_CHUNK_CLAIM..]).as_slice()
    );

    inner[10..12].copy_from_slice(&0_u16.to_le_bytes());
    let mut inner_hasher = sha2::Sha256::new();
    inner_hasher.update(&inner[..16]);
    inner_hasher.update(&inner[C11_HEADER..]);
    inner[16..C11_HEADER].copy_from_slice(&inner_hasher.finalize());
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    std::fs::write(path, media).unwrap();
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
