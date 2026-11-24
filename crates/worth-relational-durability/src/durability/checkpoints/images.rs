use contracts::config::data::policies::{AdjacencyBackend, AdjacencyPolicy};
use contracts::durability::data::recovery_errors::DurabilityError;
use contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::PartitionCheckpointImage;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::partition::adjacency::AdjacencySet;
use worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::{
    EntityRecordKind, RelationRecordKind,
};

use super::aspect_state_images::CheckpointAspectContractCatalog;

pub mod arena_images;
pub mod bitset_image;

pub(super) use arena_images::ImageReadmission;
use arena_images::{arena_from_image, arena_to_image};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop as Stop;

pub(crate) fn partition_to_image_with_contracts(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    partition: PartitionState,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
) -> Result<PartitionCheckpointImage, DurabilityError> {
    Ok(PartitionCheckpointImage {
        partition_id: partition.partition_id,
        entity_arena: arena_to_image::<EntityRecordKind>(
            lane,
            partition.entity_arena,
            catalog,
            contracts,
        )?,
        relation_arena: arena_to_image::<RelationRecordKind>(
            lane,
            partition.relation_arena,
            catalog,
            contracts,
        )?,
        adjacency: partition
            .adjacency
            .into_entries()
            .map(
                |(slot, adjacency)| worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::DurableAdjacencyEntry {
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
                |(slot, adjacency)| worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::DurableAdjacencyEntry {
                    slot: slot as u64,
                    relations: adjacency.ids(),
                    structural_revisions: adjacency.structural_revisions(),
                },
            )
            .collect(),
    })
}

pub(crate) fn partition_from_image(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    image: PartitionCheckpointImage,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
) -> Result<PartitionState, DurabilityError> {
    partition_from_image_using(
        lane,
        image,
        catalog,
        contracts,
        &ImageReadmission {
            request: None,
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
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    image: PartitionCheckpointImage,
    catalog: &AspectContractPlanCatalog,
    contracts: &CheckpointAspectContractCatalog,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PartitionState, Stop> {
    contracts::execution::request_custody::check_request(request).map_err(Stop::Request)?;
    let admission = ImageReadmission {
        request: Some(request),
    };
    partition_from_image_using(lane, image, catalog, contracts, &admission)
}
fn partition_from_image_using(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
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
            lane,
            image.partition_id,
            image.entity_arena,
            catalog,
            contracts,
            admission,
        )?,
        relation_arena: arena_from_image::<RelationRecordKind>(
            lane,
            image.partition_id,
            image.relation_arena,
            catalog,
            contracts,
            admission,
        )?,
        adjacency: adjacency_from_image(lane, image.adjacency, admission, &policy)?,
        reverse_adjacency: adjacency_from_image(lane, image.reverse_adjacency, admission, &policy)?,
        adjacency_policy: policy,
    })
}
fn adjacency_from_image(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    entries: Vec<worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::DurableAdjacencyEntry>,
    admission: &ImageReadmission<'_, '_, '_>,
    policy: &AdjacencyPolicy,
) -> Result<worth_relational_storage::internal_port::storage::partition::sparse_adjacency_table::SparseAdjacencyTable, Stop>{
    let mut table = worth_relational_storage::internal_port::storage::partition::sparse_adjacency_table::SparseAdjacencyTable::default();
    for entry in entries {
        admission.charge(1)?;
        match admission.request {
            Some(request) => {
                let value = AdjacencySet::checkpoint_for_request(
                    lane,
                    entry.relations,
                    entry.structural_revisions,
                    request,
                )?;
                *table.ensure_for_request(entry.slot as usize, policy, request)? = value;
            }
            None => {
                *table.ensure(entry.slot as usize, policy) =
                    AdjacencySet::compressed_from_checkpoint(
                        lane,
                        entry.relations,
                        entry.structural_revisions,
                    );
            }
        }
    }
    Ok(table)
}
