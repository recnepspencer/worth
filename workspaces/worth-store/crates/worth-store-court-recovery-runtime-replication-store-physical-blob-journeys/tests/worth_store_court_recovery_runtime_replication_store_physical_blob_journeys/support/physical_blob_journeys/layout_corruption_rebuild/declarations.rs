#![cfg(all())]

pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path, std::time::Duration};

pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use worth_store_contracts::DurableArtifactFamilyId;
pub(crate) use worth_store_physical_format::PersistedRecordIdentity;
pub(crate) use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::DeferredDerivedRetirementCause,
    worth_store::physical_runtime::LayoutRebuildLimits,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest,
    worth_store::physical_runtime::PhysicalIndexPointKey,
    worth_store::physical_runtime::PhysicalLayoutDenial,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_integrity::PhysicalIntegrityObservationOutcome,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) const PAGE_BYTES: usize = 16 * 1024;
pub(crate) const C5_HEADER: usize = 48;

pub(crate) fn selected_record_ids(
    serving: &ServingPhysicalRuntime,
) -> Vec<PersistedRecordIdentity> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(64).unwrap())
                .with_payload_limit(
                    worth_store::physical_runtime::RecordByteLimit::new(1).unwrap(),
                ),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    let mut selected = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        selected.extend(batch.records().iter().map(|row| {
            worth_store_physical_format::PersistedRecordIdentity::new(
                row.record_id().allocation_epoch(),
                row.record_id().ordinal(),
            )
            .unwrap()
        }));
        if batch.is_complete() {
            break;
        }
    }
    selected
}

pub(crate) fn index_identity(record: PersistedRecordIdentity) -> String {
    format!(
        "index-record:{}:{:016x}",
        record
            .allocation_epoch()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        record.ordinal()
    )
}

pub(crate) fn corrupt_selected_catalog_leaf(root: &Path, report: &serde_json::Value) -> String {
    let roots = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["family"] == "btree_node"
                && row["index_family"] == "blob_catalog"
                && row.get("expected_point_page_touches").is_some()
        })
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 1, "one selected catalog root: {report}");
    let row = roots[0];
    assert_eq!(row["outcome"]["posture"], "intact");
    let path = root.join(row["path"].as_str().unwrap());
    let offset = row["range"]["offset"].as_u64().unwrap() as usize;
    let length = row["range"]["length"].as_u64().unwrap() as usize;
    let mut media = std::fs::read(&path).unwrap();
    let frame_start = offset / PAGE_BYTES * PAGE_BYTES;
    assert!(frame_start + PAGE_BYTES <= media.len());
    assert_eq!(&media[frame_start..frame_start + 8], b"WRC5FRM\0");
    assert_eq!(media[frame_start + 8], 3, "selected inline page");
    assert_eq!(&media[offset..offset + 8], b"WRC11BTN");
    assert_eq!(media[offset + 9], 1, "catalog root is a leaf");
    assert_eq!(
        u16::from_le_bytes(media[offset + 12..offset + 14].try_into().unwrap()),
        1
    );
    assert!(length > 104 && offset + length <= frame_start + PAGE_BYTES);
    media[offset + length - 1] ^= 0x01;
    let frame = &mut media[frame_start..frame_start + PAGE_BYTES];
    let checksum = crc32c(&frame[..44], &frame[C5_HEADER..]);
    frame[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    std::fs::write(path, media).unwrap();
    row["identity"].as_str().unwrap().to_owned()
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
