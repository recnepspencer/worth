//! Reconstruct region members only under the checkpoint boundary capability.
use super::*;

pub(super) fn restore_images(
    lane: &worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    images: &[PartitionCheckpointImage],
    schema: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    kinds: &worth_relational_storage::internal_port::storage::residency::checkpoint_kind_images::CheckpointRegionKindImages,
) -> Result<
    BTreeMap<
        worth_relational_contracts::internal_port::identity::data::primitives::PartitionId,
        PartitionState,
    >,
    RelationalRecoveryStop,
> {
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())?;
    let mut partitions = BTreeMap::new();
    for image in images {
        let partition =
            partition_from_image(lane, image.clone(), schema.aspect_plans(), &contracts)?;
        if partitions
            .insert(partition.partition_id, partition)
            .is_some()
        {
            return Err(corrupt("duplicate partition image").into());
        }
    }
    kinds
        .restore(lane, execution, &mut partitions)
        .map_err(|cause| {
            crate::durability::checkpoints::kind_reconstruction::stop("kind reconstruction", cause)
        })?;
    Ok(partitions)
}

pub(super) fn capture_partitions(
    lane: &worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partitions: &BTreeMap<
        worth_relational_contracts::internal_port::identity::data::primitives::PartitionId,
        PartitionState,
    >,
    schema: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    schema_digest: [u8; 32],
    symbols: &worth_relational_contracts::internal_port::symbols::data::string_interner::StringInterner,
    members: &mut BTreeMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> Result<Vec<RelationalCheckpointRegion>, RelationalRecoveryStop> {
    partitions
        .values()
        .map(|partition| {
            let (commitment, _) = PartitionContentCommitment::capture(
                lane, execution, partition, symbols,
            )
            .map_err(|error| {
                crate::durability::checkpoints::kind_reconstruction::stop(
                    "region commitment",
                    worth_relational_storage::internal_port::storage::residency::content_capture::restore_failure(
                        partition.partition_id,
                        error,
                    ),
                )
            })?;
            let captured = crate::storage::residency::region_capture::capture_region_members(
                lane,
                execution,
                partition,
                &commitment,
                schema,
                schema_digest,
            )
            .map_err(|cause| {
                crate::durability::checkpoints::kind_reconstruction::stop("region member", cause)
            })?;
            Ok(RelationalCheckpointRegion {
                kind_authority: captured.kind_authority,
                partition_id: partition.partition_id,
                content_digest: commitment.digest(partition.partition_id),
                directory: captured
                    .directory
                    .portable(lane, execution)
                    .map_err(|error| error.into_failure(partition.partition_id))?,
                live_counts: worth_relational_storage::internal_port::storage::residency::live_counts::LiveRecordCounts::from_partition(partition),
                entity_slots: partition.entity_arena.slot_count(),
                relation_slots: partition.relation_arena.slot_count(),
                unavailable_records: partition.entity_arena.lifecycle_counts().unavailable
                    + partition.relation_arena.lifecycle_counts().unavailable,
                head: add(members, captured.head),
                history: add(members, captured.history),
            })
        })
        .collect()
}
