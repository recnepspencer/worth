use std::collections::{BTreeMap, BTreeSet};

use crate::durability::data::{DurabilityError, PartitionCheckpointImage};
use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::branch_root_images::RestoredBranchRootImages;

pub(super) type RestoredPartitions = BTreeMap<PartitionId, PartitionState>;

/// Reuse only byte-equal partitions from a root readmitted under the same
/// schema interpretation. A divergent mirror partition is reconstructed from
/// its own image, even when every sibling partition can share the substrate.
pub(super) fn restore_partition_mirror(
    checkpoint: &crate::durability::data::DurableCheckpoint,
    branch_roots: &RestoredBranchRootImages,
    plans: &crate::schema::data::AspectContractPlanCatalog,
    contracts: &crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog,
) -> Result<RestoredPartitions, DurabilityError> {
    reject_duplicate_partition_images(&checkpoint.partition_images, "checkpoint")?;
    let mut candidates = Vec::new();
    for root_image in &checkpoint.branch_roots {
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
            .partition_images
            .iter()
            .map(|image| (image.partition_id, image))
            .collect::<BTreeMap<_, _>>();
        candidates.push((by_id, restored));
    }

    let mut reused = BTreeMap::new();
    let mut fallback = Vec::new();
    for image in &checkpoint.partition_images {
        let matching = candidates.iter().find_map(|(by_id, restored)| {
            (by_id.get(&image.partition_id).copied() == Some(image))
                .then(|| restored.get(&image.partition_id))
                .flatten()
        });
        if let Some(partition) = matching {
            reused.insert(image.partition_id, partition.clone());
        } else {
            fallback.push(image.clone());
        }
    }
    let mut reconstructed =
        restore_owned_partition_images_with_schema(fallback, plans, contracts, "checkpoint")?;
    reconstructed.extend(reused);
    crate::storage::partition::rebuild_adjacency_kind_buckets(&mut reconstructed).map_err(
        |detail| {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                detail,
            )
        },
    )?;
    Ok(reconstructed)
}

pub(super) fn restore_unique_partition_images_with_schema(
    images: &[PartitionCheckpointImage],
    plans: &crate::schema::data::AspectContractPlanCatalog,
    aspect_contracts: &crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog,
    owner: &str,
) -> Result<RestoredPartitions, DurabilityError> {
    reject_duplicate_partition_images(images, owner)?;
    restore_owned_partition_images_with_schema(images.to_vec(), plans, aspect_contracts, owner)
}

fn restore_owned_partition_images_with_schema(
    images: Vec<PartitionCheckpointImage>,
    plans: &crate::schema::data::AspectContractPlanCatalog,
    aspect_contracts: &crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog,
    owner: &str,
) -> Result<RestoredPartitions, DurabilityError> {
    reject_duplicate_partition_images(&images, owner)?;
    images
        .into_iter()
        .map(|image| {
            let partition_id = image.partition_id;
            crate::durability::checkpoints::images::partition_from_image(
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
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                format!(
                    "{owner} contains duplicate partition image `{}`",
                    image.partition_id.as_u32()
                ),
            ));
        }
    }
    Ok(())
}
