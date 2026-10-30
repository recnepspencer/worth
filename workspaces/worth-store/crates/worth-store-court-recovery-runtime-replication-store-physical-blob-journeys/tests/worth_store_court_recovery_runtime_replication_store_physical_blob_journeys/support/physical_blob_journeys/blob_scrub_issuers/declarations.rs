pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path, std::time::Duration};

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobScrubTargetFailure,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest,
    worth_store::physical_runtime::PhysicalIntegrityScrubSource,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};
pub(crate) use {
    worth_store_physical_format::BlobGenerationPublicationV1,
    worth_store_physical_format::BlobRecordKind, worth_store_physical_format::BlobTreeNodeV1,
    worth_store_physical_format::PersistedRecordIdentity,
};
pub(crate) use {
    worth_store_physical_integrity::PhysicalIntegrityObservationOutcome,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};

pub(crate) use crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named;
pub(crate) use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) const CHUNK_BYTES: usize = 64 * 1024;
pub(crate) const C5_HEADER: usize = 48;
pub(crate) const C5_EXTENT_METADATA: usize = 64;
pub(crate) const C11_HEADER: usize = 48;

pub(crate) fn assert_offline_damaged(root: &Path, record: PersistedRecordIdentity, family: &str) {
    let report = observe_closed_store_named(root, "c11-selected-issuer", family);
    let identity = format!("blob-record:{}", hex_record(record));
    let row = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == identity)
        .unwrap_or_else(|| panic!("offline observer must report selected {family} {identity}"));
    assert_eq!(row["family"], family, "{report}");
    assert_eq!(row["outcome"]["posture"], "damaged", "{report}");
}

pub(crate) fn hex_record(record: PersistedRecordIdentity) -> String {
    let mut bytes = [0_u8; 24];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn assert_damaged_selected_target(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    target: worth_store::physical_runtime::PhysicalIntegrityScrubTarget,
    record: PersistedRecordIdentity,
    kind: BlobRecordKind,
) {
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        1 << 20,
        1 << 20,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    let worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(
        window,
    ) = scrub.next_window()
    else {
        panic!("selected target must inspect one bounded window")
    };
    assert_eq!(window.scope.blob_record_identity().unwrap().0, record);
    assert_eq!(window.scope.blob_record_identity().unwrap().1, kind);
    assert!(matches!(
        window.outcome,
        worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(
            worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(_)
        )
    ));
}

pub(crate) fn mutate_selected_inner(root: &Path, kind: BlobRecordKind, mutate: impl Fn(&mut [u8])) {
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let media = std::fs::read(&path).unwrap();
        for inner_offset in 0..media.len().saturating_sub(C11_HEADER) {
            let Some(outer_offset) = inner_offset.checked_sub(C5_HEADER + C5_EXTENT_METADATA)
            else {
                continue;
            };
            if !media[inner_offset..].starts_with(b"WRC11BLB")
                || media[inner_offset + 8] != kind as u8
                || !media[outer_offset..].starts_with(b"WRC5FRM\0")
                || media[outer_offset + 8] != 4
            {
                continue;
            }
            let outer_payload = u32::from_le_bytes(
                media[outer_offset + 24..outer_offset + 28]
                    .try_into()
                    .unwrap(),
            ) as usize;
            let inner_payload = u32::from_le_bytes(
                media[inner_offset + 12..inner_offset + 16]
                    .try_into()
                    .unwrap(),
            ) as usize;
            let outer_len = C5_HEADER + outer_payload;
            let inner_len = C11_HEADER + inner_payload;
            if outer_offset + outer_len <= media.len()
                && inner_offset + inner_len <= outer_offset + outer_len
            {
                candidates.push((
                    path.clone(),
                    outer_offset,
                    outer_len,
                    inner_offset,
                    inner_len,
                ));
            }
        }
    }
    assert_eq!(
        candidates.len(),
        1,
        "one selected C.11 frame of requested kind"
    );
    let (path, outer_offset, outer_len, inner_offset, inner_len) = candidates.pop().unwrap();
    let mut media = std::fs::read(&path).unwrap();
    let frame = &mut media[inner_offset..inner_offset + inner_len];
    mutate(frame);
    let digest = sha2::Sha256::digest([&frame[..16], &frame[C11_HEADER..]].concat());
    frame[16..C11_HEADER].copy_from_slice(&digest);
    match kind {
        worth_store_physical_format::BlobRecordKind::GenerationPublished => {
            worth_store_physical_format::BlobGenerationPublicationV1::decode(frame)
                .expect("resealed publication remains C.11-valid");
        }
        worth_store_physical_format::BlobRecordKind::TreeNode => {
            worth_store_physical_format::BlobTreeNodeV1::decode(frame)
                .expect("resealed tree node remains C.11-valid");
        }
        _ => unreachable!("fixture only mutates publication or tree node"),
    }
    let outer = &mut media[outer_offset..outer_offset + outer_len];
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
