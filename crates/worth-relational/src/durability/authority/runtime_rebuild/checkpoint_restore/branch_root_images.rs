use std::collections::BTreeMap;

use crate::durability::data::{DurabilityError, DurableCheckpoint};
use crate::history::data::CommitId;
use crate::runtime::RelationalRuntime;

use super::partition_images::{
    reject_duplicate_partition_images, restore_unique_partition_images_with_schema,
    RestoredPartitions,
};
use super::root_schema_readmission::RootSchemaReadmissionCatalog;

pub(super) struct RestoredBranchRootImages {
    pub(super) partitions: BTreeMap<CommitId, RestoredPartitions>,
    pub(super) schema_authorities:
        BTreeMap<CommitId, std::sync::Arc<crate::branch::RelationalBranchRootSchemaAuthority>>,
}

pub(super) fn restore_branch_root_images(
    restored: &mut RelationalRuntime,
    checkpoint: &DurableCheckpoint,
) -> Result<RestoredBranchRootImages, DurabilityError> {
    let trace = std::env::var_os("WORTH_REOPEN_TRACE").is_some();
    let started = std::time::Instant::now();
    let mut schema_catalog = RootSchemaReadmissionCatalog::readmit(checkpoint)?;
    let mut partitions = BTreeMap::new();
    let mut schema_authorities = BTreeMap::new();
    let mut envelopes = BTreeMap::new();
    for envelope in &checkpoint.envelopes {
        let commit_id = envelope.commit.commit_id;
        if envelopes.insert(commit_id, envelope.envelope()).is_some() {
            return Err(corrupt_checkpoint(format!(
                "duplicate checkpoint commit envelope `{}`",
                commit_id.0
            )));
        }
    }
    for image in &checkpoint.branch_roots {
        let envelope = envelopes.get(&image.commit_id).ok_or_else(|| {
            corrupt_checkpoint(format!(
                "branch-root image names missing commit envelope `{}`",
                image.commit_id.0
            ))
        })?;
        let owner = format!("branch-root image `{}`", image.commit_id.0);
        reject_duplicate_partition_images(&image.partition_images, &owner)?;
        let observed_digest =
            crate::durability::data::branch_root_partition_image_digest(&image.partition_images)
                .map_err(|error| {
                    corrupt_checkpoint(format!(
                        "branch-root image `{}` integrity encoding failed: {error}",
                        image.commit_id.0
                    ))
                })?;
        if trace {
            eprintln!("relational root digest: {:?}", started.elapsed());
        }
        if observed_digest != image.partition_image_digest {
            return Err(corrupt_checkpoint(format!(
                "branch-root image `{}` partition integrity mismatch",
                image.commit_id.0
            )));
        }
        let schema_authority = schema_catalog.readmit_root(restored, image, envelope)?;
        if trace {
            eprintln!("relational root schema: {:?}", started.elapsed());
        }
        let root_contracts = crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::from_contracts(
            schema_authority.retained_aspect_contracts(),
        )?;
        let mut restored_partitions = restore_unique_partition_images_with_schema(
            &image.partition_images,
            schema_authority.aspect_plans(),
            &root_contracts,
            &owner,
        )?;
        if trace {
            eprintln!("relational root partition image: {:?}", started.elapsed());
        }
        crate::storage::partition::rebuild_adjacency_kind_buckets(&mut restored_partitions)
            .map_err(|detail| {
                corrupt_checkpoint(format!(
                    "branch-root image `{}` adjacency recovery failed: {detail}",
                    image.commit_id.0
                ))
            })?;
        if trace {
            eprintln!("relational root adjacency: {:?}", started.elapsed());
        }
        if partitions
            .insert(image.commit_id, restored_partitions)
            .is_some()
        {
            return Err(corrupt_checkpoint(format!(
                "duplicate branch-root image `{}`",
                image.commit_id.0
            )));
        }
        schema_authorities.insert(image.commit_id, schema_authority);
    }
    Ok(RestoredBranchRootImages {
        partitions,
        schema_authorities,
    })
}

fn corrupt_checkpoint(detail: String) -> DurabilityError {
    DurabilityError::new(
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
        detail,
    )
}
