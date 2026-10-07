use std::{fs, num::NonZeroU64, path::Path, time::Duration};

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, BlobScrubTargetFailure,
    ManagedPhysicalIntegrityScrubProgress, ManagedPhysicalIntegrityScrubRequest,
    PhysicalIntegrityScrubSource, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobRecordKind, BlobTreeNodeV1, PersistedRecordIdentity,
};
use worth_store_physical_integrity::{
    PhysicalIntegrityObservationOutcome, PhysicalIntegrityRejection,
};

use super::blob_ingest_process::observe_closed_store_named;
use super::fixture::{
    admitted_blob_scope, placement, serving_from_initialization, serving_from_open,
};

const CHUNK_BYTES: usize = 64 * 1024;
const C5_HEADER: usize = 48;
const C5_EXTENT_METADATA: usize = 64;
const C11_HEADER: usize = 48;

#[test]
fn selected_catalog_issues_publication_before_decoding_damaged_target() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.scrub.publication.issuer");
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let (object, generation, original_record) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (2 * CHUNK_BYTES) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        ingest.push(&vec![0x41; CHUNK_BYTES]).unwrap();
        ingest.push(&vec![0x42; CHUNK_BYTES]).unwrap();
        let published = ingest.finish().unwrap();
        let target = blobs
            .scrub_publication_target(object.bytes(), published.generation().sequence())
            .unwrap();
        let record = target.scope().blob_record_identity().unwrap().0;
        assert_eq!(
            target.source(),
            PhysicalIntegrityScrubSource::SelectedRecord(record)
        );
        assert!(matches!(
            blobs.scrub_publication_target([9; 16], published.generation().sequence()),
            Err(BlobScrubTargetFailure::Selection(_))
        ));
        drop(blobs);
        serving.close();
        (object, published.generation().sequence(), record)
    };
    // The C.11 frame remains internally valid after mutation. Only the
    // protected catalog key disagrees with its object field.
    mutate_selected_inner(
        directory.path(),
        BlobRecordKind::GenerationPublished,
        |frame| {
            frame[C11_HEADER + 32] ^= 1;
        },
    );
    assert_offline_damaged(
        directory.path(),
        original_record,
        "blob_generation_publication",
    );
    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let target = blobs
        .scrub_publication_target(object.bytes(), generation)
        .unwrap();
    assert_eq!(
        target.scope().blob_record_identity().unwrap().0,
        original_record
    );
    assert!(matches!(
        blobs.scrub_publication_target([9; 16], generation),
        Err(BlobScrubTargetFailure::Selection(_))
    ));
    assert_damaged_selected_target(
        &serving,
        target,
        original_record,
        BlobRecordKind::GenerationPublished,
    );
}

#[test]
fn selected_publication_issues_root_tree_target_before_decoding_wrong_index() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.scrub.tree.issuer");
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let (published, original_record) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (2 * CHUNK_BYTES) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        ingest.push(&vec![0x52; CHUNK_BYTES]).unwrap();
        ingest.push(&vec![0x53; CHUNK_BYTES]).unwrap();
        let published = ingest.finish().unwrap();
        let target = blobs
            .scrub_tree_node_target(published, &scope, 0, 0)
            .unwrap();
        let record = target.scope().blob_record_identity().unwrap().0;
        assert!(matches!(
            blobs.scrub_tree_node_target(published, &scope, (2 * CHUNK_BYTES) as u64, 0),
            Err(BlobScrubTargetFailure::Selection(_))
        ));
        drop(blobs);
        serving.close();
        (published, record)
    };
    mutate_selected_inner(directory.path(), BlobRecordKind::TreeNode, |frame| {
        // C.11 occurrence index is outside the canonical content digest but
        // inside the encoded frame/root digest.
        frame[C11_HEADER + 32] ^= 1;
    });
    assert_offline_damaged(directory.path(), original_record, "blob_tree_node");
    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let target = blobs
        .scrub_tree_node_target(published, &scope, 0, 0)
        .unwrap();
    assert_eq!(
        target.scope().blob_record_identity().unwrap().0,
        original_record
    );
    assert_damaged_selected_target(&serving, target, original_record, BlobRecordKind::TreeNode);
}

fn assert_offline_damaged(root: &Path, record: PersistedRecordIdentity, family: &str) {
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

fn hex_record(record: PersistedRecordIdentity) -> String {
    let mut bytes = [0_u8; 24];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn assert_damaged_selected_target(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    target: worth_store::physical_runtime::PhysicalIntegrityScrubTarget,
    record: PersistedRecordIdentity,
    kind: BlobRecordKind,
) {
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
        panic!("selected target must inspect one bounded window")
    };
    assert_eq!(window.scope.blob_record_identity().unwrap().0, record);
    assert_eq!(window.scope.blob_record_identity().unwrap().1, kind);
    assert!(matches!(
        window.outcome,
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(_))
    ));
}

fn mutate_selected_inner(root: &Path, kind: BlobRecordKind, mutate: impl Fn(&mut [u8])) {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let media = fs::read(&path).unwrap();
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
    let mut media = fs::read(&path).unwrap();
    let frame = &mut media[inner_offset..inner_offset + inner_len];
    mutate(frame);
    let digest = Sha256::digest([&frame[..16], &frame[C11_HEADER..]].concat());
    frame[16..C11_HEADER].copy_from_slice(&digest);
    match kind {
        BlobRecordKind::GenerationPublished => {
            BlobGenerationPublicationV1::decode(frame)
                .expect("resealed publication remains C.11-valid");
        }
        BlobRecordKind::TreeNode => {
            BlobTreeNodeV1::decode(frame).expect("resealed tree node remains C.11-valid");
        }
        _ => unreachable!("fixture only mutates publication or tree node"),
    }
    let outer = &mut media[outer_offset..outer_offset + outer_len];
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    fs::write(path, media).unwrap();
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
