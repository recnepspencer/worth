//! Independent descriptive projection of the actual emitted checkpoint directory.
use serde::Deserialize;
use worth_relational::facade::{
    branch::RelationalBranchRootDescriptor,
    durability::{
        RelationalDurableMemberLocator, RelationalDurableRecoverySource, RelationalNativeCheckpoint,
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
    rmp_serde::from_slice::<Wire>(
        checkpoint
            .bytes()
            .strip_prefix(b"WORTH-REL-CHK24\n")
            .unwrap(),
    )
    .unwrap()
    .manifest
}
