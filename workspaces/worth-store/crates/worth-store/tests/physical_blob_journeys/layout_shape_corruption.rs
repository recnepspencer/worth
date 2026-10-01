#![cfg(feature = "certification-test-authority")]

use std::{fs, num::NonZeroU64, path::Path, time::Duration};

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits,
    ManagedPhysicalIntegrityScrubProgress, ManagedPhysicalIntegrityScrubRequest,
    PhysicalIndexPointKey, PhysicalLayoutDenial, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{BTreeNodeCellV1, BTreeNodeKind, BTreeNodeV1};
use worth_store_physical_integrity::{
    PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityObservationOutcome,
    PhysicalIntegrityRejection,
};

use super::{
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const PAGE_BYTES: usize = 16 * 1024;
const C5_HEADER: usize = 48;

#[test]
fn resealed_c9_valid_wrong_catalog_cell_width_is_damaged_to_store_and_offline_scrub() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.catalog.shape.corrupt");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let (object, published) = {
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
        ingest.push(&[0x41]).unwrap();
        ingest.push(&[0x42]).unwrap();
        let published = ingest.finish().unwrap();
        drop(blobs);
        serving.close();
        (object, published)
    };
    let before = observe_closed_store_named(root.path(), "c11-width-corruption", "before");
    let identity = reshape_selected_catalog_leaf(root.path(), &before);
    let after = observe_closed_store_named(root.path(), "c11-width-corruption", "damaged");
    let row = after["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == identity)
        .expect("independent selected walk finds same leaf");
    assert_eq!(row["family"], "btree_node");
    assert_eq!(row["outcome"]["posture"], "damaged", "{after}");

    let serving = serving_from_open(root.path());
    let key =
        PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap();
    let layouts = serving.layouts().unwrap();
    let btree = layouts.btree(DurableArtifactFamilyId::BlobCatalog).unwrap();
    assert!(matches!(
        btree.point(key),
        Err(PhysicalLayoutDenial::NodeFamilyMismatch)
    ));
    let target = btree
        .damaged_node_scrub_target()
        .unwrap()
        .expect("registered-family shape denial issues selected diagnostic target");
    let target_record = target.scope().btree_node_identity().unwrap().0;
    assert_eq!(index_identity(target_record), identity);
    drop(btree);
    drop(layouts);
    let request = ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        1 << 20,
        1 << 20,
        Duration::from_secs(30),
    )
    .unwrap();
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    let ManagedPhysicalIntegrityScrubProgress::WindowInspected(window) = scrub.next_window() else {
        panic!("selected malformed catalog node must be inspected")
    };
    assert_eq!(window.scope.btree_node_identity().unwrap().0, target_record);
    assert!(matches!(
        window.outcome,
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(
            localization
        )) if localization.scope().btree_node_identity().unwrap().0 == target_record
            && localization.cause() == PhysicalDamageCause::MalformedStructure
            && localization.field() == Some(PhysicalFormatField::Payload)
    ));
    assert_eq!(window.validation_counters.inspected_frames(), 1);
    assert_eq!(window.validation_counters.rejected_frames(), 1);
    assert_eq!(window.validation_counters.intact_frames(), 0);
    assert_eq!(window.counters.completed_windows, 1);
    assert_eq!(window.counters.validated_windows, 0);
    assert_eq!(window.counters.damaged_windows, 1);
}

fn reshape_selected_catalog_leaf(root: &Path, report: &serde_json::Value) -> String {
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
    let mut media = fs::read(&path).unwrap();
    let frame_start = offset / PAGE_BYTES * PAGE_BYTES;
    assert_eq!(&media[frame_start..frame_start + 8], b"WRC5FRM\0");
    assert_eq!(media[frame_start + 8], 3);
    assert!(offset + length <= frame_start + PAGE_BYTES);
    let original = BTreeNodeV1::decode(&media[offset..offset + length]).unwrap();
    assert_eq!(original.kind(), BTreeNodeKind::Leaf);
    assert_eq!(original.family_code(), 1);
    assert_eq!(original.cells().len(), 1);
    let cell = &original.cells()[0];
    assert_eq!(cell.key().len(), 24);
    let mut value = cell.leaf_value().unwrap().to_vec();
    assert_eq!(value.len(), 24);
    value.push(cell.key()[23]);
    let replacement = BTreeNodeV1::leaf(
        1,
        vec![BTreeNodeCellV1::leaf(cell.key()[..23].to_vec(), value)],
        original.previous_sibling(),
        original.next_sibling(),
    )
    .unwrap()
    .encode(PAGE_BYTES - C5_HEADER)
    .unwrap();
    assert_eq!(replacement.len(), length);
    assert!(
        BTreeNodeV1::decode(&replacement).is_ok(),
        "C.9 node remains valid"
    );
    media[offset..offset + length].copy_from_slice(&replacement);
    let outer = &mut media[frame_start..frame_start + PAGE_BYTES];
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    fs::write(path, media).unwrap();
    row["identity"].as_str().unwrap().to_owned()
}

fn index_identity(record: worth_store_physical_format::PersistedRecordIdentity) -> String {
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
