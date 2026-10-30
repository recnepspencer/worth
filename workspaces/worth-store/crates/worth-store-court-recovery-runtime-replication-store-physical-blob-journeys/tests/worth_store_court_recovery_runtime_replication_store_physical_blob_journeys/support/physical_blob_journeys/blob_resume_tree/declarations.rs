pub(crate) use {
    std::fs, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration,
    std::time::Instant,
};

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobIngestSession,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobResumeFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::BlobTreeNodeKind, worth_store_physical_format::BlobTreeNodeV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::marker_path,
    crate::physical_blob_journeys::blob_crash::resume_token_path,
    crate::physical_blob_journeys::blob_crash::write_marker,
    crate::physical_blob_journeys::blob_crash::SCOPE_KEY,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const CHUNK: usize = 64 * 1024;

/// The gate is installed only after both chunks selected. `finish` writes its
/// partial leaf first; the child is killed after that leaf's root replacement,
/// before it can start the generation-publication mutation.
pub(crate) fn park_after_leaf_root(
    serving: &ServingPhysicalRuntime,
    ingest: BlobIngestSession<'_>,
    root: &Path,
    object: [u8; 16],
    session: [u8; 16],
) -> ! {
    let gate = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterRootReplacement,
    );
    let marker = marker_path(root);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
            while std::time::Instant::now() < deadline {
                if gate.await_arrival() {
                    write_marker(&marker, object, session);
                    return;
                }
            }
            panic!("partial leaf never reached selected-root replacement");
        });
        let _ = ingest.finish();
        panic!("generation publication escaped selected-leaf pause before kill");
    })
}

pub(crate) fn reseal_selected_leaf_index(root: &Path, record: PhysicalRecordId, old_frame: &[u8]) {
    const C5_HEADER: usize = 48;
    const EXTENT_METADATA: usize = 64;
    const C11_HEADER: usize = 48;
    let mut locations = Vec::new();
    for entry in std::fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        if bytes.len() < C5_HEADER + EXTENT_METADATA + old_frame.len() {
            continue;
        }
        for inner_offset in C5_HEADER + EXTENT_METADATA..=bytes.len() - old_frame.len() {
            if &bytes[inner_offset..inner_offset + 8] != b"WRC11BLB" || bytes[inner_offset + 8] != 3
            {
                continue;
            }
            let outer_offset = inner_offset - C5_HEADER - EXTENT_METADATA;
            let outer = &bytes[outer_offset..];
            if &outer[..8] != b"WRC5FRM\0" || outer[8] != 4 {
                continue;
            }
            let payload_len = u32::from_le_bytes(outer[24..28].try_into().unwrap()) as usize;
            let frame_len = C5_HEADER + payload_len;
            if outer_offset + frame_len > bytes.len()
                || payload_len != EXTENT_METADATA + old_frame.len()
                || &outer[C5_HEADER..C5_HEADER + 16] != record.allocation_epoch().as_slice()
                || u64::from_le_bytes(outer[C5_HEADER + 16..C5_HEADER + 24].try_into().unwrap())
                    != record.ordinal()
                || u32::from_le_bytes(outer[C5_HEADER + 56..C5_HEADER + 60].try_into().unwrap())
                    as usize
                    != old_frame.len()
            {
                continue;
            }
            assert_eq!(
                &bytes[inner_offset..inner_offset + old_frame.len()],
                old_frame
            );
            locations.push((path.clone(), outer_offset, frame_len));
        }
    }
    assert_eq!(
        locations.len(),
        1,
        "exact selected leaf has one C5 arena frame"
    );
    let (path, outer_offset, frame_len) = locations.pop().unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    let outer = &mut bytes[outer_offset..outer_offset + frame_len];
    {
        let inner = &mut outer[C5_HEADER + EXTENT_METADATA..];
        assert_eq!(&inner[..8], b"WRC11BLB");
        assert_eq!(inner[8], 3, "only the real blob tree node may be changed");
        assert_eq!(u64::from_le_bytes(inner[80..88].try_into().unwrap()), 0);
        inner[80..88].copy_from_slice(&1_u64.to_le_bytes());
        let mut hash = sha2::Sha256::new();
        hash.update(&inner[..16]);
        hash.update(&inner[C11_HEADER..]);
        inner[16..C11_HEADER].copy_from_slice(&hash.finalize());
        let worth_store_physical_format::BlobRecordV1::TreeNode(decoded) =
            decode_blob_record(inner).unwrap()
        else {
            panic!("resealed inner frame must stay a valid tree node");
        };
        assert_eq!(decoded.occurrence().index(), 1);
    }
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    std::fs::write(path, bytes).unwrap();
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

pub(crate) fn selected_nodes(
    serving: &ServingPhysicalRuntime,
    session: [u8; 16],
) -> Vec<(PhysicalRecordId, BlobTreeNodeV1)> {
    selected_blob_records(serving)
        .into_iter()
        .filter_map(|(record, bytes)| match decode_blob_record(&bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::TreeNode(node))
                if node.occurrence().session() == session =>
            {
                Some((record, node))
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn token_from_sidecar(root: &Path) -> BlobResumeToken {
    worth_store::physical_runtime::BlobResumeToken::decode(
        &std::fs::read(resume_token_path(root)).unwrap(),
    )
    .unwrap()
}

pub(crate) fn resume_limits() -> BlobResumeLimits {
    worth_store::physical_runtime::BlobResumeLimits::new(
        std::num::NonZeroU64::new(128).unwrap(),
        std::num::NonZeroU64::new(1024 * 1024).unwrap(),
    )
}

pub(crate) fn selected_root(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

pub(crate) fn assert_record_binding(actual: PhysicalRecordId, expected: PersistedRecordIdentity) {
    assert_eq!(actual.allocation_epoch(), expected.allocation_epoch());
    assert_eq!(actual.ordinal(), expected.ordinal());
}
