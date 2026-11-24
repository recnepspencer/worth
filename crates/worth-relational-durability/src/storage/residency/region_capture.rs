use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_to_image_with_contracts;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::overlay::partition_content::PartitionContentCommitment;
use worth_relational_storage::internal_port::storage::partition::adjacency_direction::AdjacencyKindBasis;
use worth_relational_storage::internal_port::storage::residency::region_capture_data::CapturedRegionMembers;
use worth_relational_storage::internal_port::storage::residency::region_image::*;
use worth_relational_storage::internal_port::storage::residency::region_schema::RelationalRegionSchema;

pub(crate) fn capture_region_members(
    lane: &ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
    schema: &dyn RelationalRegionSchema,
    schema_carrier_digest: [u8; 32],
) -> Result<CapturedRegionMembers, worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure>{
    let history_frontiers = commitment
        .history_frontiers(lane, execution, partition)
        .map_err(|error| worth_relational_storage::internal_port::storage::residency::content_capture::restore_failure(partition.partition_id, error))?;
    worth_relational_storage::internal_port::storage::residency::kind_admission::recover(
        lane,
        execution,
        partition.partition_id,
        |checkpoint, retain| {
            let mut normalized = partition.clone();
            worth_relational_storage::internal_port::storage::residency::kind_admission::normalize_value(lane, &mut normalized, checkpoint, retain)?;
            let partition = &normalized;
            let contracts =
                CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())
                    .map_err(|_| Cause::Damaged)?;
            let mut checkpoint_partition = partition.clone();
            checkpoint_partition.clear_runtime_pin_counters();
            let mut image = partition_to_image_with_contracts(
                lane,
                checkpoint_partition,
                schema.aspect_plans(),
                &contracts,
            )
            .map_err(|_| Cause::Damaged)?;
            let history = RegionHistoryImage {
                kind_projection: worth_relational_storage::internal_port::storage::residency::kind_projection::capture_controlled(
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
                entities: worth_relational_storage::internal_port::storage::residency::history_capture::split_rows(
                    &partition.entity_arena,
                    &mut image.entity_arena.metadata_history,
                )?,
                relations: worth_relational_storage::internal_port::storage::residency::history_capture::split_rows(
                    &partition.relation_arena,
                    &mut image.relation_arena.metadata_history,
                )?,
            };
            let head = RegionHeadImage {
                kind_projection: worth_relational_storage::internal_port::storage::residency::kind_projection::capture_controlled(
                    lane,
                    partition,
                    AdjacencyKindBasis::Current,
                    checkpoint,
                )?,
                history_kind_projection: history.kind_projection.clone(),
                partition: image,
                history_frontiers,
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
            let bases = history.direct_bases(lane);
            let history = RelationalRegionImage::encode(
                lane,
                checkpoint,
                partition.partition_id,
                schema_carrier_digest,
                RelationalRegionImageKind::History,
                &history,
            )?;
            let history_members =
                worth_relational_storage::internal_port::storage::residency::history_capture::directory(lane, partition, history.0, bases)?;
            let directory =
                worth_relational_storage::internal_port::storage::residency::record_directory::RegionRecordDirectory::capture(lane, partition, &mut |units, bytes| {
                    checkpoint(units, bytes).map_err(|_| worth_relational_storage::internal_port::storage::residency::record_directory::maintenance::DirectoryError::Corrupt)
                })
                .map_err(|_| Cause::IntegrityMismatch)?;
            let kind_authority = worth_relational_storage::internal_port::storage::residency::kind_commitment::RelationalRegionKindCommitment {
                head: worth_relational_storage::internal_port::storage::residency::kind_commitment::digest_controlled(
                    lane,
                    &head.kind_projection,
                    checkpoint,
                )?,
                history: worth_relational_storage::internal_port::storage::residency::kind_commitment::digest_controlled(
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
                    lane,
                    checkpoint,
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
    table: &worth_relational_storage::internal_port::storage::partition::sparse_adjacency_table::SparseAdjacencyTable,
    basis: AdjacencyKindBasis,
    checkpoint: &mut worth_relational_storage::internal_port::storage::residency::kind_projection::ProjectionCheckpoint<'_>,
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
