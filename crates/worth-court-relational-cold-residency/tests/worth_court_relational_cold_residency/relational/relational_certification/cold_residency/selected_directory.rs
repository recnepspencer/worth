//! Independent descriptive projection of the actual emitted checkpoint directory.
use serde::Deserialize;
use worth_relational::facade::{
    branch::RelationalBranchRootDescriptor,
    durability::{
        RelationalCheckpointMember, RelationalDurableMemberLocator,
        RelationalDurableRecoverySource, RelationalNativeCheckpoint,
    },
    history::{CommitId, RelationalCommitReceipt},
    identity::PartitionId,
    indexes::DerivedIndexDefinition,
    publication::PatchStreamPosition,
};

#[derive(Deserialize)]
pub(super) struct Directory {
    pub branch_cells: Vec<serde::de::IgnoredAny>,
    pub lineage: Vec<Page>,
    pub symbols: Vec<Page>,
    pub lineage_events: Vec<EventPage>,
    pub mirror: Vec<Region>,
    pub roots: Vec<Component>,
    #[serde(rename = "commits")]
    pub commit_index: CommitIndex,
    /// Every placement, decoded from the index's pages (`CHK24`, `G7`).
    #[serde(skip)]
    pub commits: Vec<CommitPlacement>,
    pub derived_indexes: RelationalDurableMemberLocator,
    pub index_definitions: Vec<DerivedIndexDefinition>,
    pub symbol_buckets: Vec<SymbolBucket>,
}
#[derive(Deserialize)]
pub(super) struct SymbolBucket {
    pub bucket: u8,
    pub pages: Vec<RelationalDurableMemberLocator>,
}
#[derive(Deserialize, Clone, Copy)]
pub(super) struct Page {
    pub first: u64,
    pub last: u64,
    pub member: RelationalDurableMemberLocator,
}
#[derive(Deserialize)]
pub(super) struct EventPage {
    pub first_node: u64,
    pub last_node: u64,
    pub event_ids: Vec<u64>,
    pub member: RelationalDurableMemberLocator,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct Region {
    pub partition_id: PartitionId,
    pub head: RelationalDurableMemberLocator,
    pub history: RelationalDurableMemberLocator,
    #[serde(default)]
    pub relation_slots: usize,
}
#[derive(Deserialize)]
pub(super) struct Component {
    pub commit_id: CommitId,
    pub regions: Vec<Region>,
}
#[derive(Deserialize)]
pub(super) struct CommitIndex {
    pub count: u64,
    pub pages: Vec<CommitPage>,
}
#[derive(Deserialize, Clone, Copy)]
pub(super) struct CommitPage {
    pub first: u64,
    pub entries: u32,
    pub member: RelationalDurableMemberLocator,
}
#[derive(Deserialize)]
pub(super) struct CommitPlacement {
    pub position: PatchStreamPosition,
    pub commit: RelationalCommitReceipt,
    pub root: RelationalBranchRootDescriptor,
}
pub(super) fn read(input: &RelationalDurableRecoverySource) -> Directory {
    read_native(input.checkpoint().unwrap().checkpoint())
}

pub(super) fn read_native(checkpoint: &RelationalNativeCheckpoint) -> Directory {
    #[derive(Deserialize)]
    struct Wire {
        manifest: Directory,
    }
    let mut directory = rmp_serde::from_slice::<Wire>(
        checkpoint
            .bytes()
            .strip_prefix(b"WORTH-REL-CHK24\n")
            .unwrap(),
    )
    .unwrap()
    .manifest;
    directory.commits = commit_pages(checkpoint, &directory.commit_index);
    directory
}

/// A commit page is a compact chunk-member frame: magic, kind tag 4, then
/// `(first, rows)` with each row in the index entry's field order.
pub(super) fn commit_pages(
    checkpoint: &RelationalNativeCheckpoint,
    index: &CommitIndex,
) -> Vec<CommitPlacement> {
    type Row = (
        serde::de::IgnoredAny,
        RelationalCommitReceipt,
        PatchStreamPosition,
        RelationalBranchRootDescriptor,
        serde::de::IgnoredAny,
        serde::de::IgnoredAny,
    );
    let mut placements = Vec::new();
    for page in &index.pages {
        let bytes = checkpoint.members().iter().find_map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } if *locator == page.member => {
                Some(bytes.clone())
            }
            _ => None,
        });
        // A carrying capture writes only its new pages; its carried prefix is
        // not decodable from this checkpoint alone.
        let Some(bytes) = bytes else {
            return Vec::new();
        };
        let frame = bytes.strip_prefix(b"WORTH-REL-CHUNK1\n").unwrap();
        assert_eq!(frame[0], 4, "commit index page tag");
        let (first, rows): (u64, Vec<Row>) = rmp_serde::from_slice(&frame[1..]).unwrap();
        assert_eq!((first, rows.len()), (page.first, page.entries as usize));
        placements.extend(rows.into_iter().map(|(_, commit, position, root, _, _)| {
            CommitPlacement {
                position,
                commit,
                root,
            }
        }));
    }
    assert_eq!(placements.len() as u64, index.count);
    placements
}
