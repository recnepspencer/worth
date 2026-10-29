//! Generations whose ingest checkpointed resume frontiers. A frontier is
//! residue of the released session: it leaves with the release, before any
//! chunk, so the head still becomes terminal and retires.

pub(crate) use {worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

pub(crate) use {crate::physical_blob_journeys::terminal_head_retirement::non_reissue::assert_release_is_the_sessions_fate, crate::physical_blob_journeys::terminal_head_retirement::non_reissue::assert_retired_identity_stays_ended, crate::physical_blob_journeys::terminal_head_retirement::non_reissue::retire_once_expired};
pub(crate) use {crate::physical_blob_journeys::terminal_head_retirement::checkpoint, crate::physical_blob_journeys::terminal_head_retirement::denied, crate::physical_blob_journeys::terminal_head_retirement::observe, crate::physical_blob_journeys::terminal_head_retirement::publication::publish_chunks, crate::physical_blob_journeys::terminal_head_retirement::release_batch, crate::physical_blob_journeys::terminal_head_retirement::settle, worth_store::physical_runtime::BlobTerminalHeadRetirementDenial as Denial, worth_store::physical_runtime::BlobTerminalHeadRetirementFailure as Failure, crate::physical_blob_journeys::terminal_head_retirement::publication::Published, crate::physical_blob_journeys::terminal_head_retirement::CHUNK};
pub(crate) use {worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records};
pub(crate) use {worth_store_testkit_store::fixture::serving_from_initialization};

/// One ingest that leaves resume frontiers selected under its publication.
pub(crate) struct Ingest {
    pub(crate) chunks: usize,
    /// The chunk counts at which the ingest checkpoints a frontier itself.
    pub(crate) frontier_prefixes: &'static [usize],
    /// The frontiers the published generation leaves selected.
    pub(crate) frontiers: usize,
    /// More checkpoints than the whole release completes: settling a batch
    /// completes one checkpoint for each record it dropped. The released
    /// session is therefore inside its resume horizon at every batch.
    pub(crate) horizon: u64,
}

/// Two chunks past the ordinal at which an ingest checkpoints a frontier by
/// itself. Its release drops 69 records.
pub(crate) const LONG: Ingest = Ingest {
    chunks: 66,
    frontier_prefixes: &[],
    frontiers: 1,
    horizon: 80,
};
/// A small generation with two frontiers, each followed by a chunk that no
/// frontier names. Its release drops 8 records.
pub(crate) const TWO_FRONTIERS: Ingest = Ingest {
    chunks: 4,
    frontier_prefixes: &[1, 3],
    frontiers: 2,
    horizon: 16,
};

/// The selected records of one session, by kind.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SessionRecords {
    pub(crate) declarations: usize,
    pub(crate) publications: usize,
    pub(crate) frontiers: usize,
    pub(crate) chunks: usize,
    pub(crate) trees: usize,
}

pub(crate) fn session_records(serving: &ServingPhysicalRuntime, published: &Published) -> SessionRecords {
    let object = published.object.bytes();
    let selected = selected_blob_records(serving);
    let records = || {
        selected
            .iter()
            .filter_map(|(_, bytes)| decode_blob_record(bytes).ok())
    };
    let session = records()
        .find_map(|record| match record {
            worth_store_physical_format::BlobRecordV1::SessionDeclared(declared) if declared.object() == object => {
                Some(declared.session())
            }
            _ => None,
        })
        .expect("the declaration of the object stays selected");
    let mut counted = crate::physical_blob_journeys::terminal_head_retirement::frontier::SessionRecords::default();
    for record in records() {
        match record {
            worth_store_physical_format::BlobRecordV1::SessionDeclared(value) if value.session() == session => {
                counted.declarations += 1;
            }
            worth_store_physical_format::BlobRecordV1::GenerationPublished(value) if value.session() == session => {
                counted.publications += 1;
            }
            worth_store_physical_format::BlobRecordV1::SessionFrontier(value) if value.session() == session => {
                counted.frontiers += 1;
            }
            worth_store_physical_format::BlobRecordV1::Chunk(value) if value.occurrence().session() == session => {
                counted.chunks += 1;
            }
            worth_store_physical_format::BlobRecordV1::TreeNode(value) if value.occurrence().session() == session => {
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
pub(crate) fn release_and_retire(ingest: &Ingest, batches: &[(u16, usize)]) {
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
                worth_store::physical_runtime::BlobTerminalHeadRetirementFailure::Denied(worth_store::physical_runtime::BlobTerminalHeadRetirementDenial::IdentityDeclarationNotExpired { .. })
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
            ..crate::physical_blob_journeys::terminal_head_retirement::frontier::SessionRecords::default()
        },
        "only the declaration of a retired identity stays selected"
    );
    serving.close();
}