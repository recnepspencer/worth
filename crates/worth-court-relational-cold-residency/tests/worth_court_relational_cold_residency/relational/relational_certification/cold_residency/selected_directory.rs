//! Independent descriptive projection of the actual emitted checkpoint directory.
use serde::Deserialize;
use worth_relational::facade::{
    durability::{RelationalDurableMemberLocator, RelationalDurableRecoverySource},
    history::CommitId,
    identity::PartitionId,
};

#[derive(Deserialize)]
pub(super) struct Directory {
    pub lineage: Vec<Page>,
    pub symbols: Vec<Page>,
    pub lineage_events: Vec<EventPage>,
    pub mirror: Vec<Region>,
    pub roots: Vec<Component>,
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
#[derive(Deserialize)]
pub(super) struct Region {
    pub partition_id: PartitionId,
    pub head: RelationalDurableMemberLocator,
    pub history: RelationalDurableMemberLocator,
}
#[derive(Deserialize)]
pub(super) struct Component {
    pub commit_id: CommitId,
    pub regions: Vec<Region>,
}
pub(super) fn read(input: &RelationalDurableRecoverySource) -> Directory {
    #[derive(Deserialize)]
    struct Wire {
        manifest: Directory,
    }
    rmp_serde::from_slice::<Wire>(
        input
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(b"WORTH-REL-CHK18\n")
            .unwrap(),
    )
    .unwrap()
    .manifest
}
