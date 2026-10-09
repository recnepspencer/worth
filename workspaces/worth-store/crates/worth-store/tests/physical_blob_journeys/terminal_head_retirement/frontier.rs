//! Generations whose ingest checkpointed resume frontiers. A frontier is
//! residue of the released session: it leaves with the release, before any
//! chunk, so the head still becomes terminal and retires.

use worth_store::physical_runtime::ServingPhysicalRuntime;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::non_reissue::{
    assert_release_is_the_sessions_fate, assert_retired_identity_stays_ended, retire_once_expired,
};
use super::{
    checkpoint, denied, observe, publish_chunks, release_batch, settle, Denial, Failure, Published,
    CHUNK,
};
use crate::blob_frontier::selected_blob_records;
use crate::fixture::serving_from_initialization;

/// One ingest that leaves resume frontiers selected under its publication.
struct Ingest {
    chunks: usize,
    /// The chunk counts at which the ingest checkpoints a frontier itself.
    frontier_prefixes: &'static [usize],
    /// The frontiers the published generation leaves selected.
    frontiers: usize,
    /// More checkpoints than the whole release completes: settling a batch
    /// completes one checkpoint for each record it dropped. The released
    /// session is therefore inside its resume horizon at every batch.
    horizon: u64,
}

/// Two chunks past the ordinal at which an ingest checkpoints a frontier by
/// itself. Its release drops 69 records.
const LONG: Ingest = Ingest {
    chunks: 66,
    frontier_prefixes: &[],
    frontiers: 1,
    horizon: 80,
};
/// A small generation with two frontiers, each followed by a chunk that no
/// frontier names. Its release drops 8 records.
const TWO_FRONTIERS: Ingest = Ingest {
    chunks: 4,
    frontier_prefixes: &[1, 3],
    frontiers: 2,
    horizon: 16,
};

/// The selected records of one session, by kind.
#[derive(Debug, Default, PartialEq, Eq)]
struct SessionRecords {
    declarations: usize,
    publications: usize,
    frontiers: usize,
    chunks: usize,
    trees: usize,
}

fn session_records(serving: &ServingPhysicalRuntime, published: &Published) -> SessionRecords {
    let object = published.object.bytes();
    let selected = selected_blob_records(serving);
    let records = || {
        selected
            .iter()
            .filter_map(|(_, bytes)| decode_blob_record(bytes).ok())
    };
    let session = records()
        .find_map(|record| match record {
            BlobRecordV1::SessionDeclared(declared) if declared.object() == object => {
                Some(declared.session())
            }
            _ => None,
        })
        .expect("the declaration of the object stays selected");
    let mut counted = SessionRecords::default();
    for record in records() {
        match record {
            BlobRecordV1::SessionDeclared(value) if value.session() == session => {
                counted.declarations += 1;
            }
            BlobRecordV1::GenerationPublished(value) if value.session() == session => {
                counted.publications += 1;
            }
            BlobRecordV1::SessionFrontier(value) if value.session() == session => {
                counted.frontiers += 1;
            }
            BlobRecordV1::Chunk(value) if value.occurrence().session() == session => {
                counted.chunks += 1;
            }
            BlobRecordV1::TreeNode(value) if value.occurrence().session() == session => {
                counted.trees += 1;
            }
            _ => {}
        }
    }
    counted
}

/// Releases the generation of `ingest` in exactly the given batches, then
/// retires its head. Each entry is the batch capacity and the records that
/// batch must drop. No chunk leaves while a frontier stays selected.
fn release_and_retire(ingest: &Ingest, batches: &[(u16, usize)]) {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let chunks: Vec<Vec<u8>> = (0..ingest.chunks)
        .map(|ordinal| {
            let mut chunk = vec![0x31; CHUNK];
            chunk[..8].copy_from_slice(&(ordinal as u64).to_le_bytes());
            chunk
        })
        .collect();
    let published = publish_chunks(&serving, &chunks, ingest.frontier_prefixes, ingest.horizon);
    let (proof, token) = (&published.proof, published.token);
    let whole = session_records(&serving, &published);
    assert_eq!(
        whole,
        SessionRecords {
            declarations: 1,
            publications: 1,
            frontiers: ingest.frontiers,
            chunks: ingest.chunks,
            trees: 1,
        },
        "the ingest checkpointed its frontiers under one root node"
    );

    // The publication is not owed payload; the frontiers, chunks and tree are.
    let mut owed = (whole.frontiers + whole.chunks + whole.trees) as u64;
    let mut first = true;
    for &(capacity, dropped) in batches {
        assert!(owed > 0, "the release was terminal before its last batch");
        let mut receipt = release_batch(&serving, proof, capacity);
        assert_eq!(receipt.dropped_records().len(), dropped);
        owed -= (dropped - usize::from(first)) as u64;
        first = false;
        assert_eq!(receipt.remaining_payload_records(), owed);
        let left = session_records(&serving, &published);
        assert_eq!((left.declarations, left.publications), (1, 0));
        assert!(
            left.frontiers == 0 || left.chunks == ingest.chunks,
            "a chunk left under a frontier that stayed selected: {left:?}"
        );
        assert_eq!((left.frontiers + left.chunks + left.trees) as u64, owed);
        assert_release_is_the_sessions_fate(&serving, token);
        settle(&serving, &mut receipt);
        assert_release_is_the_sessions_fate(&serving, token);
        let unexpired = denied(&serving, proof);
        assert!(
            matches!(
                unexpired,
                Failure::Denied(Denial::IdentityDeclarationNotExpired { .. })
            ),
            "the release outlived the resume horizon of its session: {unexpired:?}"
        );
    }
    assert_eq!(owed, 0, "the batches did not reach a terminal head");
    assert_eq!(observe(&serving).effective_heads().0, 1);

    let receipt = retire_once_expired(&serving, proof, 0x40);
    assert!(receipt.head_tree_emptied());
    assert_eq!(observe(&serving).effective_heads().0, 0);
    assert_retired_identity_stays_ended(&serving, &published);
    checkpoint(&serving, 0x82);
    assert_eq!(observe(&serving).checkpoint_heads().0, 0);
    assert_retired_identity_stays_ended(&serving, &published);
    assert_eq!(
        session_records(&serving, &published),
        SessionRecords {
            declarations: 1,
            ..SessionRecords::default()
        },
        "only the declaration of a retired identity stays selected"
    );
    serving.close();
}

/// The first batch takes the publication, the frontier and 62 chunks; the
/// second takes the last four chunks and then their root node.
#[test]
fn a_generation_with_a_resume_frontier_releases_to_a_terminal_head_and_retires() {
    release_and_retire(&LONG, &[(64, 64), (64, 5)]);
}

/// A batch too small for a chunk beside the frontier takes the frontier
/// alone, and the chunks follow in later batches.
#[test]
fn a_resume_frontier_leaves_alone_when_its_batch_has_no_room_for_a_chunk() {
    release_and_retire(&LONG, &[(1, 1), (1, 1), (64, 64), (64, 3)]);
}

/// A frontier of the released session keeps nothing: one batch with room
/// for the generation takes both frontiers, the chunk each names, the other
/// chunks and their root node.
#[test]
fn two_resume_frontiers_and_the_chunks_they_name_leave_in_one_batch() {
    release_and_retire(&TWO_FRONTIERS, &[(64, 8)]);
}

/// Batches too small for both frontiers split them: the publication, one
/// frontier alone, then the other frontier beside the first chunk.
#[test]
fn two_resume_frontiers_split_across_batches_both_leave_before_any_chunk() {
    release_and_retire(&TWO_FRONTIERS, &[(1, 1), (1, 1), (2, 2), (64, 4)]);
}
