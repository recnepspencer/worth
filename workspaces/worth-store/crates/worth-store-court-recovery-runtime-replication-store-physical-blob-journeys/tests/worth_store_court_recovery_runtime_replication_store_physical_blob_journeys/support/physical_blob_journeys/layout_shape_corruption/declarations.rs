#![cfg(all())]

pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path, std::time::Duration};

pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest, worth_store::physical_runtime::PhysicalIndexPointKey, worth_store::physical_runtime::PhysicalLayoutDenial, worth_store::physical_runtime::PhysicalMutationDeadline};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_contracts::DurableArtifactFamilyId};
pub(crate) use {worth_store_physical_format::BTreeNodeCellV1, worth_store_physical_format::BTreeNodeKind, worth_store_physical_format::BTreeNodeV1};
pub(crate) use {worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome, worth_store_physical_integrity::PhysicalIntegrityRejection};

pub(crate) use {crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const PAGE_BYTES: usize = 16 * 1024;
pub(crate) const C5_HEADER: usize = 48;

pub(crate) fn reshape_selected_catalog_leaf(root: &Path, report: &serde_json::Value) -> String {
    let rows = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["family"] == "btree_node"
                && row["index_family"] == "blob_catalog"
                && row.get("expected_point_page_touches").is_some()
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "one selected catalog root: {report}");
    let row = rows[0];
    assert_eq!(row["outcome"]["posture"], "intact");
    let path = root.join(row["path"].as_str().unwrap());
    let offset = row["range"]["offset"].as_u64().unwrap() as usize;
    let length = row["range"]["length"].as_u64().unwrap() as usize;
    let mut media = std::fs::read(&path).unwrap();
    let frame_start = offset / PAGE_BYTES * PAGE_BYTES;
    assert_eq!(&media[frame_start..frame_start + 8], b"WRC5FRM\0");
    assert_eq!(media[frame_start + 8], 3);
    assert!(offset + length <= frame_start + PAGE_BYTES);
    let original = worth_store_physical_format::BTreeNodeV1::decode(&media[offset..offset + length]).unwrap();
    assert_eq!(original.kind(), worth_store_physical_format::BTreeNodeKind::Leaf);
    assert_eq!(original.family_code(), 1);
    assert_eq!(original.cells().len(), 1);
    let cell = &original.cells()[0];
    assert_eq!(cell.key().len(), 24);
    let mut value = cell.leaf_value().unwrap().to_vec();
    assert_eq!(value.len(), 24);
    value.push(cell.key()[23]);
    let replacement = worth_store_physical_format::BTreeNodeV1::leaf(
        1,
        vec![worth_store_physical_format::BTreeNodeCellV1::leaf(cell.key()[..23].to_vec(), value)],
        original.previous_sibling(),
        original.next_sibling(),
    )
    .unwrap()
    .encode(PAGE_BYTES - C5_HEADER)
    .unwrap();
    assert_eq!(replacement.len(), length);
    assert!(
        worth_store_physical_format::BTreeNodeV1::decode(&replacement).is_ok(),
        "C.9 node remains valid"
    );
    media[offset..offset + length].copy_from_slice(&replacement);
    let outer = &mut media[frame_start..frame_start + PAGE_BYTES];
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    std::fs::write(path, media).unwrap();
    row["identity"].as_str().unwrap().to_owned()
}

pub(crate) fn index_identity(record: worth_store_physical_format::PersistedRecordIdentity) -> String {
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