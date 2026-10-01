#![cfg(feature = "certification-test-authority")]

use std::{fs, num::NonZeroU64, path::Path, time::Duration};

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, DeferredDerivedRetirementCause,
    LayoutRebuildLimits, ManagedPhysicalIntegrityScrubProgress,
    ManagedPhysicalIntegrityScrubRequest, PhysicalIndexPointKey, PhysicalLayoutDenial,
    PhysicalMutationDeadline, RecordByteLimit, RecordCountLimit, RecordScanOutcome,
    RecordScanRequest, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::PersistedRecordIdentity;
use worth_store_physical_integrity::{
    PhysicalIntegrityObservationOutcome, PhysicalIntegrityRejection,
};

use super::{
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const PAGE_BYTES: usize = 16 * 1024;
const C5_HEADER: usize = 48;

#[test]
fn damaged_catalog_leaf_denies_point_then_rebuilds_from_selected_blob_authority() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.layout.corrupt-leaf.rebuild");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let (object, published, original) = {
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
        ingest.push(&[0x31]).unwrap();
        ingest.push(&[0x72]).unwrap();
        let published = ingest.finish().unwrap();
        let key =
            PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap();
        let original = serving
            .layouts()
            .unwrap()
            .btree(DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record()
            .unwrap();
        drop(blobs);
        serving.close();
        (object, published, original)
    };

    let before = observe_closed_store_named(root.path(), "c11-leaf-rebuild", "before");
    assert_eq!(before["completeness"], "complete", "{before}");
    let old_node_identity = corrupt_selected_catalog_leaf(root.path(), &before);
    let damaged = observe_closed_store_named(root.path(), "c11-leaf-rebuild", "damaged");
    let damaged_node = damaged["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == old_node_identity)
        .expect("same selected catalog node after mutation");
    assert_eq!(damaged_node["family"], "btree_node");
    assert_eq!(damaged_node["outcome"]["posture"], "damaged", "{damaged}");

    let serving = serving_from_open(root.path());
    let key =
        PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap();
    let layouts = serving.layouts().unwrap();
    let btree = layouts.btree(DurableArtifactFamilyId::BlobCatalog).unwrap();
    assert!(matches!(
        btree.point(key),
        Err(PhysicalLayoutDenial::NodeIntegrity(_))
    ));
    assert!(btree.damaged_node_scrub_target().unwrap().is_some());
    let foreign_root = tempfile::tempdir().unwrap();
    let foreign_serving = serving_from_initialization(foreign_root.path());
    let foreign_object = foreign_serving
        .blobs()
        .unwrap()
        .issue_object_id(limits)
        .unwrap();
    let foreign_key =
        PhysicalIndexPointKey::blob_catalog(foreign_object, published.generation().sequence())
            .unwrap();
    assert!(matches!(
        btree.point(foreign_key),
        Err(PhysicalLayoutDenial::ForeignStore)
    ));
    assert!(
        btree.damaged_node_scrub_target().unwrap().is_none(),
        "a new denied request must not inherit a prior damaged-node target"
    );
    foreign_serving.close();
    assert!(matches!(
        btree.point(key),
        Err(PhysicalLayoutDenial::NodeIntegrity(_))
    ));
    let target = btree
        .damaged_node_scrub_target()
        .unwrap()
        .expect("selected D1 lookup issues exact damaged-node diagnostic target");
    let target_record = target.scope().btree_node_identity().unwrap().0;
    assert_eq!(index_identity(target_record), old_node_identity);
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
        panic!("selected catalog leaf scrub must inspect its bounded window")
    };
    assert_eq!(window.scope.btree_node_identity().unwrap().0, target_record);
    assert!(matches!(
        window.outcome,
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(
            localization
        )) if localization.scope().btree_node_identity().unwrap().0 == target_record
    ));
    assert_eq!(window.counters.damaged_windows, 1);
    drop(scrub);
    let selected_records_before_rebuild = selected_record_ids(&serving);
    let bounded = LayoutRebuildLimits::new(
        NonZeroU64::new(1_000).unwrap(),
        NonZeroU64::new(1_000).unwrap(),
    );
    let rebuilt = serving
        .layouts()
        .unwrap()
        .rebuild(
            DurableArtifactFamilyId::BlobCatalog,
            bounded,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .unwrap();
    // The receipt counts authoritative logical chunks preflighted, not the
    // later second traversal's physical validation operations.
    assert_eq!(rebuilt.validated_chunks(), 1);
    assert_eq!(
        rebuilt.scanned_records(),
        selected_records_before_rebuild.len() as u64
    );
    assert_eq!(rebuilt.deferred_derived_root_count(), 1);
    assert_eq!(rebuilt.deferred_dedupe_root(), None);
    assert_eq!(rebuilt.deferred_dedupe_cause(), None);
    assert!(matches!(
        rebuilt.deferred_catalog_cause(),
        Some(DeferredDerivedRetirementCause::NodeDamaged(_))
    ));
    let deferred_catalog = rebuilt
        .deferred_catalog_root()
        .expect("damaged old catalog root must be retained for later recovery");
    assert_eq!(index_identity(deferred_catalog), old_node_identity);
    assert!(
        selected_record_ids(&serving).contains(&deferred_catalog),
        "unproved old derived closure must remain selected, not be dropped"
    );
    assert_eq!(
        serving
            .layouts()
            .unwrap()
            .btree(DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        Some(original)
    );
    serving.close();
    let after = observe_closed_store_named(root.path(), "c11-leaf-rebuild", "rebuilt");
    assert_eq!(after["completeness"], "complete", "{after}");
    let new_node_identity = index_identity(rebuilt.catalog_root());
    let new_node = after["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["identity"] == new_node_identity)
        .expect("offline selected walk reaches replacement catalog root");
    assert_eq!(new_node["outcome"]["posture"], "intact", "{after}");
    assert!(
        !after["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["identity"] == old_node_identity),
        "offline selected-root walk must not present unreachable old residue as a live index node"
    );
    let reopened = serving_from_open(root.path());
    assert_eq!(
        reopened
            .layouts()
            .unwrap()
            .btree(DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(key)
            .unwrap()
            .selected_record(),
        Some(original)
    );
    reopened.close();
}

fn selected_record_ids(serving: &ServingPhysicalRuntime) -> Vec<PersistedRecordIdentity> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(64).unwrap())
                .with_payload_limit(RecordByteLimit::new(1).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    let mut selected = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        selected.extend(batch.records().iter().map(|row| {
            PersistedRecordIdentity::new(
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

fn index_identity(record: PersistedRecordIdentity) -> String {
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

fn corrupt_selected_catalog_leaf(root: &Path, report: &serde_json::Value) -> String {
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
    let mut media = fs::read(&path).unwrap();
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
    fs::write(path, media).unwrap();
    row["identity"].as_str().unwrap().to_owned()
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
