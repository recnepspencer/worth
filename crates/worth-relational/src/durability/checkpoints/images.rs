use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::durability::data::{DurabilityError, PartitionCheckpointImage};
use crate::schema::data::AspectContractPlanCatalog;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencySet;
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind};

use super::aspect_state_images::CheckpointAspectContractCatalog;

mod arena_images;
mod bitset_image;

use arena_images::{arena_from_image, arena_to_image};

pub(crate) fn partition_to_image(
    partition: PartitionState,
    catalog: &AspectContractPlanCatalog,
) -> Result<PartitionCheckpointImage, DurabilityError> {
    let contracts = CheckpointAspectContractCatalog::from_plans(catalog)?;
    partition_to_image_with_contracts(partition, catalog, &contracts)
}

pub(crate) fn partition_to_image_with_contracts(
    partition: PartitionState,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
) -> Result<PartitionCheckpointImage, DurabilityError> {
    Ok(PartitionCheckpointImage {
        partition_id: partition.partition_id,
        entity_arena: arena_to_image::<EntityRecordKind>(
            partition.entity_arena,
            catalog,
            contracts,
        )?,
        relation_arena: arena_to_image::<RelationRecordKind>(
            partition.relation_arena,
            catalog,
            contracts,
        )?,
        adjacency: partition
            .adjacency
            .into_entries()
            .map(
                |(slot, adjacency)| crate::durability::data::DurableAdjacencyEntry {
                    slot: slot as u64,
                    relations: adjacency.ids(),
                    structural_revisions: adjacency.structural_revisions(),
                },
            )
            .collect(),
        reverse_adjacency: partition
            .reverse_adjacency
            .into_entries()
            .map(
                |(slot, adjacency)| crate::durability::data::DurableAdjacencyEntry {
                    slot: slot as u64,
                    relations: adjacency.ids(),
                    structural_revisions: adjacency.structural_revisions(),
                },
            )
            .collect(),
    })
}

pub(crate) fn partition_from_image(
    image: PartitionCheckpointImage,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
) -> Result<PartitionState, DurabilityError> {
    Ok(PartitionState {
        partition_id: image.partition_id,
        adjacency_policy: AdjacencyPolicy {
            backend: AdjacencyBackend::CompressedFanoutAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: arena_from_image::<EntityRecordKind>(
            image.partition_id,
            image.entity_arena,
            catalog,
            contracts,
        )?,
        relation_arena: arena_from_image::<RelationRecordKind>(
            image.partition_id,
            image.relation_arena,
            catalog,
            contracts,
        )?,
        adjacency: crate::storage::partition::SparseAdjacencyTable::from_entries(
            image.adjacency.into_iter().map(|entry| {
                (
                    entry.slot as usize,
                    AdjacencySet::compressed_from_checkpoint(
                        entry.relations,
                        entry.structural_revisions,
                    ),
                )
            }),
        ),
        reverse_adjacency: crate::storage::partition::SparseAdjacencyTable::from_entries(
            image.reverse_adjacency.into_iter().map(|entry| {
                (
                    entry.slot as usize,
                    AdjacencySet::compressed_from_checkpoint(
                        entry.relations,
                        entry.structural_revisions,
                    ),
                )
            }),
        ),
    })
}
