use std::{collections::BTreeMap, fs, num::NonZeroU64, path::Path};

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits,
    PhysicalMutationDeadline, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::blob_ingest_process::{observe_closed_store_named, observed_without_damage};
use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

const CHUNK: usize = 64 * 1024;
const RUN: &str = "c11-blob-unread-record";
const ARENA: &str = "families/records/arenas/arena-0000000000000001.data";
const C5_MAGIC: &[u8] = b"WRC5FRM\0";
const C5_HEADER: usize = 48;
const C5_EXTENT_MANIFEST: u8 = 6;
const C5_EXTENT_CHUNK: u8 = 4;
const RECORD_IDENTITY: usize = 24;

/// A frame of the last record published that the observer cannot read.
#[derive(Clone, Copy)]
enum DamagedFrame {
    ExtentManifest,
    FirstExtentChunk,
}

#[test]
fn a_record_whose_extent_manifest_is_damaged_keeps_its_row_and_excuses_no_other() {
    a_record_with_a_damaged_frame_keeps_its_row_and_excuses_no_other(
        "damaged-manifest",
        DamagedFrame::ExtentManifest,
    );
}

#[test]
fn a_record_whose_first_extent_chunk_is_damaged_keeps_its_row_and_excuses_no_other() {
    a_record_with_a_damaged_frame_keeps_its_row_and_excuses_no_other(
        "damaged-first-chunk",
        DamagedFrame::FirstExtentChunk,
    );
}

/// A record whose extent manifest or first extent chunk is damaged was routed
/// and not read. The observer still names it: the record keeps its row, of the
/// family its route declares and with that frame's damage. Nothing claims the
/// last publication, so no other record is anything but intact.
fn a_record_with_a_damaged_frame_keeps_its_row_and_excuses_no_other(
    scenario: &str,
    damaged: DamagedFrame,
) {
    let directory = tempfile::tempdir().unwrap();
    let clean = two_objects_of_one_chunk(directory.path(), scenario);
    let last_published =
        flip_a_frame_of_the_last_record_without_resealing(directory.path(), damaged);

    let report = observe_closed_store_named(directory.path(), RUN, scenario);
    assert_eq!(report["completeness"], "complete", "{report}");
    let observed = blob_records(report["artifacts"].as_array().unwrap());
    assert_eq!(
        observed.keys().collect::<Vec<_>>(),
        clean.keys().collect::<Vec<_>>(),
        "every routed blob record keeps one row"
    );
    assert_eq!(
        clean[&last_published]["family"],
        "blob_generation_publication"
    );
    for (record, row) in &observed {
        assert_eq!(row["family"], clean[record]["family"], "{row}");
        if *record == last_published {
            assert_eq!(row["outcome"]["posture"], "damaged", "{row}");
            assert_eq!(row["outcome"]["cause"], "checksum_mismatch", "{row}");
        } else {
            assert_eq!(row["outcome"]["posture"], "intact", "{row}");
        }
    }
}

/// The observer reads a file once. An arena that is a second name of a file
/// it has already seen is not reinspected, so no record in it was read. Each
/// record keeps its row, of the family its route declares, as not reinspected:
/// none is absent, and none is damaged for what it claims of another.
#[test]
fn records_of_an_arena_that_is_not_reinspected_keep_their_rows_undamaged() {
    let directory = tempfile::tempdir().unwrap();
    let clean = two_objects_of_one_chunk(directory.path(), "aliased-arena");
    fs::hard_link(
        directory.path().join(ARENA),
        directory.path().join("namespace/arena-alias"),
    )
    .expect("same-volume hard-link fixture must be supported");

    let artifacts = observed_without_damage(directory.path(), RUN, "aliased-arena");
    let observed = blob_records(&artifacts);
    assert_eq!(
        observed.keys().collect::<Vec<_>>(),
        clean.keys().collect::<Vec<_>>(),
        "every routed blob record keeps one row"
    );
    for (record, row) in &observed {
        assert_eq!(row["family"], clean[record]["family"], "{row}");
        assert_eq!(row["outcome"]["posture"], "unknown", "{row}");
        assert_eq!(
            row["outcome"]["reason"], "physical_alias_not_reinspected",
            "{row}"
        );
    }
}

/// Publishes two objects of the same one chunk, closes the store and returns
/// the blob records of its clean observation: two declarations, one chunk,
/// one reuse claim, two tree nodes and two publications, all intact.
fn two_objects_of_one_chunk(root: &Path, scenario: &str) -> BTreeMap<String, serde_json::Value> {
    let scope = admitted_blob_scope("c11.blob.unread-record.scope");
    let serving = serving_from_initialization(root);
    publish_one_chunk(&serving, &scope);
    publish_one_chunk(&serving, &scope);
    serving.close();

    let scenario = format!("{scenario}-clean");
    let clean = blob_records(&observed_without_damage(root, RUN, &scenario));
    assert_eq!(clean.len(), 8, "{clean:?}");
    for row in clean.values() {
        assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    }
    clean
}

/// The blob record rows of an observation, by record. A record has one row.
fn blob_records(artifacts: &[serde_json::Value]) -> BTreeMap<String, serde_json::Value> {
    let mut rows = BTreeMap::new();
    for artifact in artifacts {
        let Some(record) = artifact["identity"]
            .as_str()
            .and_then(|identity| identity.strip_prefix("blob-record:"))
        else {
            continue;
        };
        let prior = rows.insert(record.to_owned(), artifact.clone());
        assert!(prior.is_none(), "two rows of one record: {artifact}");
    }
    rows
}

/// Flips one payload byte of a frame of the last record published and leaves
/// the frame's checksum as it was. That record's extent manifest is the last
/// one in the arena, and its first extent chunk is the next frame after it.
/// Returns the record as the observer spells it.
fn flip_a_frame_of_the_last_record_without_resealing(root: &Path, damaged: DamagedFrame) -> String {
    let path = root.join(ARENA);
    let mut media = fs::read(&path).unwrap();
    let frames = (0..media.len().saturating_sub(C5_HEADER + RECORD_IDENTITY))
        .filter(|&offset| media[offset..].starts_with(C5_MAGIC))
        .collect::<Vec<_>>();
    let kind = |frame: usize| media[frame + C5_MAGIC.len()];
    let manifest = frames
        .iter()
        .rposition(|&frame| kind(frame) == C5_EXTENT_MANIFEST)
        .expect("the arena holds an extent manifest");
    let record = media[frames[manifest] + C5_HEADER..][..RECORD_IDENTITY]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let frame = match damaged {
        DamagedFrame::ExtentManifest => frames[manifest],
        DamagedFrame::FirstExtentChunk => {
            let chunk = *frames
                .get(manifest + 1)
                .expect("an extent chunk follows the last extent manifest");
            assert_eq!(kind(chunk), C5_EXTENT_CHUNK);
            chunk
        }
    };
    media[frame + C5_HEADER + RECORD_IDENTITY] ^= 1;
    fs::write(path, media).unwrap();
    record
}

fn publish_one_chunk(serving: &ServingPhysicalRuntime, scope: &AdmittedBlobScope) {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let declaration = BlobIngestDeclaration::new(
        blobs.issue_object_id(limits).unwrap(),
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(300_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, limits)
        .unwrap();
    ingest.push(&[63; CHUNK / 2]).unwrap();
    ingest.push(&[63; CHUNK / 2]).unwrap();
    ingest.finish().unwrap();
}
