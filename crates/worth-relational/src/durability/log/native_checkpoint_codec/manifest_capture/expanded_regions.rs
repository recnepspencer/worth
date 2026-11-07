//! Reconstruct region members only under the checkpoint boundary capability.
use super::*;

pub(super) fn restore_images(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    images: &[PartitionCheckpointImage],
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    kinds: &crate::storage::residency::CheckpointRegionKindImages,
) -> Result<BTreeMap<crate::identity::data::PartitionId, PartitionState>, RelationalRecoveryStop> {
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())?;
    let mut partitions = BTreeMap::new();
    for image in images {
        let partition = partition_from_image(image.clone(), schema.aspect_plans(), &contracts)?;
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
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partitions: &BTreeMap<crate::identity::data::PartitionId, PartitionState>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    schema_digest: [u8; 32],
    symbols: &crate::symbols::data::StringInterner,
    members: &mut BTreeMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> Result<Vec<RelationalCheckpointRegion>, RelationalRecoveryStop> {
    partitions
        .values()
        .map(|partition| {
            let (commitment, _) = PartitionContentCommitment::capture(partition, symbols, None)
                .map_err(|error| corrupt(format!("region commitment: {error:?}")))?;
            let captured = crate::storage::residency::capture_region_members(
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
                directory: captured.directory.portable(),
                live_counts: crate::storage::residency::LiveRecordCounts::from_partition(partition),
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
