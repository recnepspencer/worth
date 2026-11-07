use super::region_image::*;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_to_image_with_contracts;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::substrate::maintenance::ReconstructionLane;

pub(crate) struct CapturedRegionMembers {
    pub(crate) history_members:
        std::sync::Arc<crate::durability::data::RelationalHistoryMemberDirectory>,
    pub(crate) directory: std::sync::Arc<super::RegionRecordDirectory>,
    pub(crate) kind_authority: super::RelationalRegionKindCommitment,
    pub(crate) head: (RelationalDurableMemberLocator, Vec<u8>),
    pub(crate) history: (RelationalDurableMemberLocator, Vec<u8>),
}

pub(crate) fn capture_region_members(
    lane: &ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
    schema: &RelationalBranchRootSchemaAuthority,
    schema_carrier_digest: [u8; 32],
) -> Result<CapturedRegionMembers, super::RelationalResidencyDemandFailure> {
    super::kind_admission::recover(
        lane,
        execution,
        partition.partition_id,
        |checkpoint, retain| {
            let mut normalized = partition.clone();
            super::kind_admission::normalize_value(lane, &mut normalized, checkpoint, retain)?;
            let partition = &normalized;
            let contracts =
                CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())
                    .map_err(|_| Cause::Damaged)?;
            let mut checkpoint_partition = partition.clone();
            checkpoint_partition.clear_runtime_pin_counters();
            let mut image = partition_to_image_with_contracts(
                checkpoint_partition,
                schema.aspect_plans(),
                &contracts,
            )
            .map_err(|_| Cause::Damaged)?;
            let history = RegionHistoryImage {
                kind_projection: super::kind_projection::capture_controlled(
                    lane,
                    partition,
                    AdjacencyKindBasis::Historical,
                    checkpoint,
                )?,
                adjacency_kinds: capture_kinds(
                    lane,
                    &partition.adjacency,
                    AdjacencyKindBasis::Historical,
                    checkpoint,
                )?,
                reverse_adjacency_kinds: capture_kinds(
                    lane,
                    &partition.reverse_adjacency,
                    AdjacencyKindBasis::Historical,
                    checkpoint,
                )?,
                entities: super::history_capture::split_rows(
                    &partition.entity_arena,
                    &mut image.entity_arena.metadata_history,
                )?,
                relations: super::history_capture::split_rows(
                    &partition.relation_arena,
                    &mut image.relation_arena.metadata_history,
                )?,
            };
            let head = RegionHeadImage {
                kind_projection: super::kind_projection::capture_controlled(
                    lane,
                    partition,
                    AdjacencyKindBasis::Current,
                    checkpoint,
                )?,
                history_kind_projection: history.kind_projection.clone(),
                partition: image,
                history_frontiers: commitment
                    .history_frontiers(partition)
                    .ok_or(Cause::Damaged)?,
                adjacency_kinds: capture_kinds(
                    lane,
                    &partition.adjacency,
                    AdjacencyKindBasis::Current,
                    checkpoint,
                )?,
                reverse_adjacency_kinds: capture_kinds(
                    lane,
                    &partition.reverse_adjacency,
                    AdjacencyKindBasis::Current,
                    checkpoint,
                )?,
            };
            let bases = history.direct_bases();
            let history = RelationalRegionImage::encode(
                partition.partition_id,
                schema_carrier_digest,
                RelationalRegionImageKind::History,
                &history,
            )?;
            let history_members = super::history_capture::directory(partition, history.0, bases)?;
            let directory =
                super::RegionRecordDirectory::capture(lane, partition, &mut |units, bytes| {
                    checkpoint(units, bytes).map_err(|_| super::RecordDirectoryError::Corrupt)
                })
                .map_err(|_| Cause::IntegrityMismatch)?;
            let kind_authority = super::RelationalRegionKindCommitment {
                head: super::kind_commitment::digest_controlled(
                    lane,
                    &head.kind_projection,
                    checkpoint,
                )?,
                history: super::kind_commitment::digest_controlled(
                    lane,
                    &head.history_kind_projection,
                    checkpoint,
                )?,
            };
            Ok(CapturedRegionMembers {
                directory,
                history_members,
                kind_authority,
                head: RelationalRegionImage::encode(
                    partition.partition_id,
                    schema_carrier_digest,
                    RelationalRegionImageKind::Head,
                    &head,
                )?,
                history,
            })
        },
    )
}

pub(super) fn capture_kinds(
    lane: &ReconstructionLane<'_>,
    table: &crate::storage::partition::SparseAdjacencyTable,
    basis: AdjacencyKindBasis,
    checkpoint: &mut super::kind_projection::ProjectionCheckpoint<'_>,
) -> Result<Vec<RegionAdjacencyKinds>, Cause> {
    let entries = table.iter();
    let bytes = entries
        .len()
        .checked_mul(size_of::<RegionAdjacencyKinds>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Cause::MemoryExhausted)?;
    checkpoint(0, bytes)?;
    let mut answer = Vec::new();
    answer
        .try_reserve_exact(entries.len())
        .map_err(|_| Cause::MemoryExhausted)?;
    for (&slot, adjacency) in entries {
        checkpoint(1, 0)?;
        let mut kinds = adjacency.checkpoint_kind_memberships(lane, basis, checkpoint)?;
        // Compact empty kinds in place while preserving the ordered image.
        let mut retained = 0;
        for index in 0..kinds.len() {
            checkpoint(1, 0)?;
            if !kinds[index].1.is_empty() {
                kinds.swap(retained, index);
                retained += 1;
            }
        }
        kinds.truncate(retained);
        answer.push(RegionAdjacencyKinds {
            slot: slot as u64,
            kinds,
        });
    }
    Ok(answer)
}
