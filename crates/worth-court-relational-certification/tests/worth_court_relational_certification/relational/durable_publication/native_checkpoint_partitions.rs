//! Read exact checkpoint Head rows by their actual committed root and locator.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        PartitionCheckpointImage, RelationalCheckpointMember, RelationalDurableMemberLocator,
        RelationalNativeCheckpoint,
    },
    history::CommitId,
    identity::PartitionId,
    runtime::RelationalRuntime,
};

#[derive(Deserialize)]
struct Wire {
    manifest: Manifest,
}
#[derive(Deserialize)]
struct Manifest {
    roots: Vec<Root>,
}
#[derive(Deserialize)]
struct Root {
    commit_id: CommitId,
    regions: Vec<Region>,
}
#[derive(Deserialize)]
struct Region {
    partition_id: PartitionId,
    head: RelationalDurableMemberLocator,
}
#[derive(Deserialize)]
struct RegionHead {
    #[serde(rename = "kind_projection")]
    _kind_projection: serde::de::IgnoredAny,
    #[serde(rename = "history_kind_projection")]
    _history_kind_projection: serde::de::IgnoredAny,
    partition: PartitionCheckpointImage,
    #[serde(rename = "history_frontiers")]
    _history_frontiers: serde::de::IgnoredAny,
    #[serde(rename = "adjacency_kinds")]
    _adjacency_kinds: serde::de::IgnoredAny,
    #[serde(rename = "reverse_adjacency_kinds")]
    _reverse_adjacency_kinds: serde::de::IgnoredAny,
}

pub(super) fn original(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> RelationalNativeCheckpoint {
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
execution,
execution.allocation_policy(),
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
        .unwrap()
}

pub(super) fn partition_image(
    checkpoint: &RelationalNativeCheckpoint,
    commit: CommitId,
    partition: PartitionId,
) -> PartitionCheckpointImage {
    let wire: Wire = rmp_serde::from_slice(
        checkpoint
            .bytes()
            .strip_prefix(b"WORTH-REL-CHK23\n")
            .unwrap(),
    )
    .unwrap();
    let root = wire
        .manifest
        .roots
        .iter()
        .find(|root| root.commit_id == commit)
        .expect("the actual committed root was emitted");
    let region = root
        .regions
        .iter()
        .find(|region| region.partition_id == partition)
        .unwrap();
    let member = checkpoint
        .members()
        .iter()
        .find(|member| member.locator() == region.head)
        .unwrap();
    let RelationalCheckpointMember::New { bytes, .. } = member else {
        panic!("the original Embedded checkpoint owns its Head bytes");
    };
    assert_eq!(bytes.len() as u64, region.head.length);
    assert_eq!(
        <[u8; 32]>::from(Sha256::digest(bytes.as_ref())),
        region.head.sha256
    );
    let framed = bytes.as_ref().strip_prefix(b"WORTH-REL-REG7\n").unwrap();
    assert_eq!(&framed[..2], &7_u16.to_be_bytes());
    assert_eq!(&framed[2..6], &partition.as_u32().to_be_bytes());
    assert_eq!(framed[38], 0);
    let decoded: RegionHead = rmp_serde::from_slice(&framed[39..]).unwrap();
    assert_eq!(decoded.partition.partition_id, partition);
    decoded.partition
}

pub(super) fn physical_index(slots: &[u64], slot: u64) -> usize {
    if slots.is_empty() {
        usize::try_from(slot).unwrap()
    } else {
        slots.binary_search(&slot).unwrap()
    }
}
