use std::collections::BTreeSet;
use std::sync::Arc;

use super::region_image::*;
use super::{RelationalColdRegion, ResidentRegion};
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_from_image, partition_to_image_with_contracts,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::{AdjacencyKindBasis, SparseAdjacencyTable};

pub(super) fn load_head(
    cold: &RelationalColdRegion,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Cause> {
    check_member_limit(cold.head.length, limit)?;
    let bytes = cold.source.read_member(cold.head, limit)?;
    let frame = RelationalRegionImage::decode(
        &bytes,
        cold.head,
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
    )?;
    let head: RegionHeadImage =
        rmp_serde::from_slice(&frame.payload).map_err(|_| Cause::Damaged)?;
    if head.partition.partition_id != cold.partition_id {
        return Err(Cause::Damaged);
    }
    let contracts = contracts(cold)?;
    let mut partition =
        partition_from_image(head.partition, cold.schema.aspect_plans(), &contracts)
            .map_err(|_| Cause::Damaged)?;
    install_kinds(
        &mut partition.adjacency,
        head.adjacency_kinds,
        AdjacencyKindBasis::Current,
    )?;
    install_kinds(
        &mut partition.reverse_adjacency,
        head.reverse_adjacency_kinds,
        AdjacencyKindBasis::Current,
    )?;
    let commitment = PartitionContentCommitment::capture_head(
        &partition,
        &cold.symbols,
        &head.history_frontiers,
    )
    .ok_or(Cause::Damaged)?;
    verify_leaf(cold, &partition, &commitment)?;
    Ok(Arc::new(ResidentRegion {
        partition,
        commitment,
    }))
}

pub(super) fn load_full(
    cold: &RelationalColdRegion,
    head: &ResidentRegion,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Cause> {
    check_member_limit(cold.history.length, limit)?;
    let bytes = cold.source.read_member(cold.history, limit)?;
    let frame = RelationalRegionImage::decode(
        &bytes,
        cold.history,
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
    )?;
    let history: RegionHistoryImage =
        rmp_serde::from_slice(&frame.payload).map_err(|_| Cause::Damaged)?;
    let contracts = contracts(cold)?;
    let mut image = partition_to_image_with_contracts(
        head.partition.clone(),
        cold.schema.aspect_plans(),
        &contracts,
    )
    .map_err(|_| Cause::Damaged)?;
    if history.entities.len() != image.entity_arena.metadata_history.len()
        || history.relations.len() != image.relation_arena.metadata_history.len()
    {
        return Err(Cause::Damaged);
    }
    image.entity_arena.metadata_history = history.entities;
    image.relation_arena.metadata_history = history.relations;
    let mut partition = partition_from_image(image, cold.schema.aspect_plans(), &contracts)
        .map_err(|_| Cause::Damaged)?;
    partition.adjacency = head.partition.adjacency.clone();
    partition.reverse_adjacency = head.partition.reverse_adjacency.clone();
    install_kinds(
        &mut partition.adjacency,
        history.adjacency_kinds,
        AdjacencyKindBasis::Historical,
    )?;
    install_kinds(
        &mut partition.reverse_adjacency,
        history.reverse_adjacency_kinds,
        AdjacencyKindBasis::Historical,
    )?;
    let (commitment, _) = PartitionContentCommitment::capture(&partition, &cold.symbols, None)
        .map_err(|_| Cause::Damaged)?;
    verify_leaf(cold, &partition, &commitment)?;
    Ok(Arc::new(ResidentRegion {
        partition,
        commitment,
    }))
}

fn contracts(cold: &RelationalColdRegion) -> Result<CheckpointAspectContractCatalog, Cause> {
    CheckpointAspectContractCatalog::from_contracts(cold.schema.retained_aspect_contracts())
        .map_err(|_| Cause::Damaged)
}

fn check_member_limit(length: u64, limit: u64) -> Result<(), Cause> {
    if length > limit {
        return Err(Cause::LimitExceeded { length, limit });
    }
    Ok(())
}

fn verify_leaf(
    cold: &RelationalColdRegion,
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
) -> Result<(), Cause> {
    if commitment.digest(cold.partition_id) != cold.content_digest
        || partition.entity_arena.slot_count() != cold.entity_slots
        || partition.relation_arena.slot_count() != cold.relation_slots
        || partition.entity_arena.lifecycle_counts().unavailable
            + partition.relation_arena.lifecycle_counts().unavailable
            != cold.unavailable_records
    {
        return Err(Cause::IntegrityMismatch);
    }
    Ok(())
}

fn install_kinds(
    table: &mut SparseAdjacencyTable,
    entries: Vec<RegionAdjacencyKinds>,
    basis: AdjacencyKindBasis,
) -> Result<(), Cause> {
    let mut slots = BTreeSet::new();
    for entry in entries {
        if !slots.insert(entry.slot) {
            return Err(Cause::Damaged);
        }
        let adjacency = table
            .get_mut(usize::try_from(entry.slot).map_err(|_| Cause::Damaged)?)
            .ok_or(Cause::Damaged)?;
        let mut relations = BTreeSet::new();
        for (kind, ids) in entry.kinds {
            for id in ids {
                if !relations.insert(id) {
                    return Err(Cause::Damaged);
                }
                if matches!(basis, AdjacencyKindBasis::Historical) {
                    adjacency.index_historical_kind(kind, id);
                } else {
                    adjacency.index_current_kind(kind, id);
                }
            }
        }
        if matches!(basis, AdjacencyKindBasis::Current)
            && relations != adjacency.current_ids().iter().copied().collect()
        {
            return Err(Cause::Damaged);
        }
    }
    if slots.len() != table.iter().count() {
        return Err(Cause::Damaged);
    }
    Ok(())
}
