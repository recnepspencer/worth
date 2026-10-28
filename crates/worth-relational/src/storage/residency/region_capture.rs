use super::region_image::*;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::partition_to_image_with_contracts;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;

pub(crate) struct CapturedRegionMembers {
    pub(crate) history_members:
        std::sync::Arc<crate::durability::data::RelationalHistoryMemberDirectory>,
    pub(crate) kind_authority: super::RelationalRegionKindCommitment,
    pub(crate) head: (RelationalDurableMemberLocator, Vec<u8>),
    pub(crate) history: (RelationalDurableMemberLocator, Vec<u8>),
}

pub(crate) fn capture_region_members(
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
    schema: &RelationalBranchRootSchemaAuthority,
    schema_carrier_digest: [u8; 32],
) -> Result<CapturedRegionMembers, Cause> {
    let mut normalized = partition.clone();
    super::rebuild_region_history_kind_buckets(&mut normalized)?;
    let partition = &normalized;
    let contracts =
        CheckpointAspectContractCatalog::from_contracts(schema.retained_aspect_contracts())
            .map_err(|_| Cause::Damaged)?;
    let mut checkpoint_partition = partition.clone();
    checkpoint_partition.clear_runtime_pin_counters();
    let mut image =
        partition_to_image_with_contracts(checkpoint_partition, schema.aspect_plans(), &contracts)
            .map_err(|_| Cause::Damaged)?;
    let history = RegionHistoryImage {
        kind_projection: super::kind_projection::capture(
            partition,
            AdjacencyKindBasis::Historical,
        )?,
        adjacency_kinds: capture_kinds(&partition.adjacency, AdjacencyKindBasis::Historical),
        reverse_adjacency_kinds: capture_kinds(
            &partition.reverse_adjacency,
            AdjacencyKindBasis::Historical,
        ),
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
        kind_projection: super::kind_projection::capture(partition, AdjacencyKindBasis::Current)?,
        history_kind_projection: history.kind_projection.clone(),
        partition: image,
        history_frontiers: commitment
            .history_frontiers(partition)
            .ok_or(Cause::Damaged)?,
        adjacency_kinds: capture_kinds(&partition.adjacency, AdjacencyKindBasis::Current),
        reverse_adjacency_kinds: capture_kinds(
            &partition.reverse_adjacency,
            AdjacencyKindBasis::Current,
        ),
    };
    let bases = history.direct_bases();
    let history = RelationalRegionImage::encode(
        partition.partition_id,
        schema_carrier_digest,
        RelationalRegionImageKind::History,
        &history,
    )?;
    let history_members = super::history_capture::directory(partition, history.0, bases)?;
    Ok(CapturedRegionMembers {
        history_members,
        kind_authority: super::RelationalRegionKindCommitment {
            head: super::kind_commitment::digest(&head.kind_projection)?,
            history: super::kind_commitment::digest(&head.history_kind_projection)?,
        },
        head: RelationalRegionImage::encode(
            partition.partition_id,
            schema_carrier_digest,
            RelationalRegionImageKind::Head,
            &head,
        )?,
        history,
    })
}

pub(super) fn capture_kinds(
    table: &crate::storage::partition::SparseAdjacencyTable,
    basis: AdjacencyKindBasis,
) -> Vec<RegionAdjacencyKinds> {
    table
        .iter()
        .map(|(&slot, adjacency)| RegionAdjacencyKinds {
            slot: slot as u64,
            kinds: adjacency
                .checkpoint_kind_memberships(basis)
                .into_iter()
                .filter(|(_, ids)| !ids.is_empty())
                .collect(),
        })
        .collect()
}
