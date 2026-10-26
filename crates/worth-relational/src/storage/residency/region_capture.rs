use super::region_image::*;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_to_image_with_contracts;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;

pub(crate) struct CapturedRegionMembers {
    pub(crate) head: (RelationalDurableMemberLocator, Vec<u8>),
    pub(crate) history: (RelationalDurableMemberLocator, Vec<u8>),
}

pub(crate) fn capture_region_members(
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
    schema: &RelationalBranchRootSchemaAuthority,
    schema_carrier_digest: [u8; 32],
) -> Result<CapturedRegionMembers, Cause> {
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())
            .map_err(|_| Cause::Damaged)?;
    let mut checkpoint_partition = partition.clone();
    checkpoint_partition.clear_runtime_pin_counters();
    let mut image =
        partition_to_image_with_contracts(checkpoint_partition, schema.aspect_plans(), &contracts)
            .map_err(|_| Cause::Damaged)?;
    let history = RegionHistoryImage {
        entities: image
            .entity_arena
            .metadata_history
            .iter_mut()
            .map(|rows| {
                let current = rows.pop();
                let earlier = std::mem::take(rows);
                rows.extend(current);
                earlier
            })
            .collect(),
        relations: image
            .relation_arena
            .metadata_history
            .iter_mut()
            .map(|rows| {
                let current = rows.pop();
                let earlier = std::mem::take(rows);
                rows.extend(current);
                earlier
            })
            .collect(),
    };
    let head = RegionHeadImage {
        partition: image,
        history_frontiers: commitment
            .history_frontiers(partition)
            .ok_or(Cause::Damaged)?,
        adjacency_kinds: capture_kinds(&partition.adjacency, AdjacencyKindBasis::Current),
        reverse_adjacency_kinds: capture_kinds(
            &partition.reverse_adjacency,
            AdjacencyKindBasis::Current,
        ),
    };
    Ok(CapturedRegionMembers {
        head: RelationalRegionImage::encode(
            partition.partition_id,
            schema_carrier_digest,
            RelationalRegionImageKind::Head,
            &head,
        )?,
        history: RelationalRegionImage::encode(
            partition.partition_id,
            schema_carrier_digest,
            RelationalRegionImageKind::History,
            &history,
        )?,
    })
}

fn capture_kinds(
    table: &crate::storage::partition::SparseAdjacencyTable,
    basis: AdjacencyKindBasis,
) -> Vec<RegionAdjacencyKinds> {
    table
        .iter()
        .map(|(&slot, adjacency)| RegionAdjacencyKinds {
            slot: slot as u64,
            kinds: adjacency.checkpoint_kind_memberships(basis),
        })
        .collect()
}
