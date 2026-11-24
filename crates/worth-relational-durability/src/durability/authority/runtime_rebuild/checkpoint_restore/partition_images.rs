use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port as contracts;

use crate::durability::data::CheckpointRestoreWork;
use contracts::durability::data::recovery_errors::DurabilityError;
use contracts::identity::data::primitives::PartitionId;
use worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::PartitionCheckpointImage;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;

use super::branch_root_images::RestoredBranchRootImages;

pub(super) type RestoredPartitions = BTreeMap<PartitionId, PartitionState>;

pub(super) struct CheckpointMirrorImages<'a> {
    pub(super) partitions: &'a [PartitionCheckpointImage],
    pub(super) kinds: &'a worth_relational_storage::internal_port::storage::residency::checkpoint_kind_images::CheckpointRegionKindImages,
}

pub(super) struct CheckpointRootPartitions<'a> {
    pub(super) commit_id: contracts::history::data::coordinates::CommitId,
    pub(super) partitions: &'a [PartitionCheckpointImage],
}

/// Reuse only byte-equal partitions from a root readmitted under the same
/// schema interpretation. A divergent mirror partition is reconstructed from
/// its own image, even when every sibling partition can share the substrate.
pub(super) fn restore_partition_mirror<'image>(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    images: CheckpointMirrorImages<'_>,
    root_images: impl IntoIterator<Item = CheckpointRootPartitions<'image>>,
    branch_roots: &RestoredBranchRootImages,
    plans: &contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
    contracts: &crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog,
    work: &mut CheckpointRestoreWork,
) -> Result<RestoredPartitions, contracts::durability::data::recovery_stop::RelationalRecoveryStop>
{
    reject_duplicate_partition_images(images.partitions, "checkpoint")?;
    let mut candidates = Vec::new();
    for root_image in root_images {
        let Some(schema) = branch_roots.schema_authorities.get(&root_image.commit_id) else {
            continue;
        };
        if schema.aspect_plans() != plans {
            continue;
        }
        let root_contracts = crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::from_contracts(
            schema.retained_aspect_contracts(),
        )?;
        if &root_contracts != contracts {
            continue;
        }
        let Some(restored) = branch_roots.partitions.get(&root_image.commit_id) else {
            continue;
        };
        let by_id = root_image
            .partitions
            .iter()
            .map(|image| (image.partition_id, image))
            .collect::<BTreeMap<_, _>>();
        candidates.push((by_id, restored));
    }

    let mut reused = BTreeMap::new();
    let mut fallback = Vec::new();
    for image in images.partitions {
        work.mirror_partition_images_examined += 1;
        let matching = candidates.iter().find_map(|(by_id, restored)| {
            (by_id.get(&image.partition_id).copied() == Some(image))
                .then(|| restored.get(&image.partition_id))
                .flatten()
        });
        if let Some(partition) = matching {
            reused.insert(image.partition_id, partition.clone());
            work.mirror_partitions_reused += 1;
        } else {
            fallback.push(image.clone());
            work.mirror_partitions_reconstructed += 1;
        }
    }
    let mut reconstructed =
        restore_owned_partition_images_with_schema(lane, fallback, plans, contracts, "checkpoint")?;
    reconstructed.extend(reused);
    images
        .kinds
        .restore(lane, execution, &mut reconstructed)
        .map_err(|cause| {
            crate::durability::checkpoints::kind_reconstruction::stop(
                "mirror kind reconstruction",
                cause,
            )
        })?;
    Ok(reconstructed)
}

pub(super) fn restore_unique_partition_images_with_schema(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    images: &[PartitionCheckpointImage],
    plans: &contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
    aspect_contracts: &crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog,
    owner: &str,
) -> Result<RestoredPartitions, DurabilityError> {
    reject_duplicate_partition_images(images, owner)?;
    restore_owned_partition_images_with_schema(
        lane,
        images.to_vec(),
        plans,
        aspect_contracts,
        owner,
    )
}

fn restore_owned_partition_images_with_schema(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    images: Vec<PartitionCheckpointImage>,
    plans: &contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
    aspect_contracts: &crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog,
    owner: &str,
) -> Result<RestoredPartitions, DurabilityError> {
    reject_duplicate_partition_images(&images, owner)?;
    images
        .into_iter()
        .map(|image| {
            let partition_id = image.partition_id;
            crate::durability::checkpoints::images::partition_from_image(
                lane,
                image,
                plans,
                aspect_contracts,
            )
            .map(|partition| (partition_id, partition))
        })
        .collect()
}

pub(super) fn reject_duplicate_partition_images(
    images: &[PartitionCheckpointImage],
    owner: &str,
) -> Result<(), DurabilityError> {
    let mut seen = BTreeSet::new();
    for image in images {
        if !seen.insert(image.partition_id) {
            return Err(DurabilityError::new(
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
                format!(
                    "{owner} contains duplicate partition image `{}`",
                    image.partition_id.as_u32()
                ),
            ));
        }
    }
    Ok(())
}
