//! One move-only result retains every real member and intrinsic directory owner.
mod dictionary_pages;
mod lineage_pages;
use super::super::symbols::OriginalSymbolDictionary;
use super::{array_stop, encode_indexes, map_stop, OriginalCheckpointLineage};
use crate::durability::{data::*, medium::RelationalDurableMemberLocator};
use crate::runtime::OriginalCheckpointIndexes;
use worth_execution::{ExecutionImmutableBytes, ExecutionOrderedMap, ExecutionRequest};
use RelationalNativeCheckpointCaptureDenial as Denial;
pub(in crate::durability::authority::checkpoint_complete) struct CheckpointMetadataMembers {
    pub(in crate::durability::authority::checkpoint_complete) symbols: CheckpointPages,
    pub(in crate::durability::authority::checkpoint_complete) symbol_buckets:
        CheckpointSymbolBuckets,
    pub(in crate::durability::authority::checkpoint_complete) lineage: CheckpointPages,
    pub(in crate::durability::authority::checkpoint_complete) lineage_events:
        CheckpointLineageEventPages,
    pub(in crate::durability::authority::checkpoint_complete) derived_indexes:
        RelationalDurableMemberLocator,
    pub(in crate::durability::authority::checkpoint_complete) members:
        ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
}
pub(in crate::durability::authority::checkpoint_complete) fn capture_metadata_members(
    dictionary: &OriginalSymbolDictionary<'_>,
    lineage: &OriginalCheckpointLineage<'_>,
    indexes: &OriginalCheckpointIndexes,
    prior: Option<&RelationalCheckpointManifest>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CheckpointMetadataMembers, Denial> {
    let mut members = ExecutionOrderedMap::default();
    let (symbols, symbol_buckets) = dictionary_pages::capture(dictionary, request, &mut members)?;
    let (nodes, events) = lineage_pages::capture(lineage, prior, request, &mut members)?;
    let derived_indexes = add(
        &mut members,
        encode_indexes(indexes, request).map_err(stop)?,
        request,
    )?;
    Ok(CheckpointMetadataMembers {
        symbols,
        symbol_buckets,
        lineage: nodes,
        lineage_events: events,
        derived_indexes,
        members,
    })
}
fn add(
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    (locator, bytes): (RelationalDurableMemberLocator, ExecutionImmutableBytes),
    request: &ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableMemberLocator, Denial> {
    if members.get(&locator).is_none() {
        members
            .insert_cloned_native_for_request(
                &locator,
                &RelationalCheckpointMember::New { locator, bytes },
                *request,
            )
            .map_err(map_stop)?;
    }
    Ok(locator)
}
fn stop(cause: RelationalRecoveryStop) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(cause)
}
