use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::durability::data::{DurabilityError, PartitionCheckpointImage};
use crate::schema::data::AspectContractPlanCatalog;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencySet;
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind};

use super::aspect_state_images::CheckpointAspectContractCatalog;

mod arena_images;
mod bitset_image;

use crate::durability::data::RelationalRecoveryStop as Stop;
pub(super) use arena_images::ImageReadmission;
use arena_images::{arena_from_image, arena_to_image};

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
    partition_from_image_using(
        image,
        catalog,
        contracts,
        &ImageReadmission {
            request: None,
            contract_bound: 0,
        },
    )
    .map_err(|cause| match cause {
        Stop::Durability(cause) => cause,
        Stop::Request(_) | Stop::Allocation(_) | Stop::AdmissionImage(_) | Stop::Content(_) => {
            unreachable!("ordinary image conversion has no request, content demand or admission-image operation")
        }
    })
}
pub(crate) fn partition_from_image_for_request(
    image: PartitionCheckpointImage,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PartitionState, Stop> {
    let admission = ImageReadmission {
        request: Some(request),
        contract_bound: contracts.max_clone_bound(request)?,
    };
    partition_from_image_using(image, catalog, contracts, &admission)
}
fn partition_from_image_using(
    image: PartitionCheckpointImage,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<PartitionState, Stop> {
    let policy = AdjacencyPolicy {
        backend: AdjacencyBackend::CompressedFanoutAdjacency,
        small_degree_inline_capacity: 4,
    };
    Ok(PartitionState {
        partition_id: image.partition_id,
        relation_overlay_is_sparse: false,
        entity_arena: arena_from_image::<EntityRecordKind>(
            image.partition_id,
            image.entity_arena,
            catalog,
            contracts,
            admission,
        )?,
        relation_arena: arena_from_image::<RelationRecordKind>(
            image.partition_id,
            image.relation_arena,
            catalog,
            contracts,
            admission,
        )?,
        adjacency: adjacency_from_image(image.adjacency, admission, &policy)?,
        reverse_adjacency: adjacency_from_image(image.reverse_adjacency, admission, &policy)?,
        adjacency_policy: policy,
    })
}
fn adjacency_from_image(
    entries: Vec<crate::durability::data::DurableAdjacencyEntry>,
    admission: &ImageReadmission<'_, '_, '_>,
    policy: &AdjacencyPolicy,
) -> Result<crate::storage::partition::SparseAdjacencyTable, Stop> {
    let mut table = crate::storage::partition::SparseAdjacencyTable::default();
    for entry in entries {
        admission.charge(1)?;
        match admission.request {
            Some(request) => {
                let value = AdjacencySet::checkpoint_for_request(
                    entry.relations,
                    entry.structural_revisions,
                    request,
                )?;
                *table.ensure_for_request(entry.slot as usize, policy, request)? = value;
            }
            None => {
                *table.ensure(entry.slot as usize, policy) =
                    AdjacencySet::compressed_from_checkpoint(
                        entry.relations,
                        entry.structural_revisions,
                    );
            }
        }
    }
    Ok(table)
}
