use crate::validation::engine::request::{PreparedRelationEndpointKey, PreparedRelationPairKey};
use std::borrow::Cow;
mod candidate_entities;
use candidate_entities::record_candidate_entity;
use std::collections::{BTreeMap, BTreeSet};

use crate::schema::data::LoweredCardinalityMinimumContract;
use crate::storage::substrate::VersionedRelationMetadata;
use crate::transactions::data::EntityReference;

use super::super::super::context::InvariantExecutionContext;
use super::super::super::state_view::InvariantStateView;
use super::minimum_current_index::CurrentVersionMinimumIndex;

#[derive(Default)]
pub(super) struct VisibleRelationCountSnapshot<'scope> {
    pub(super) source_counts: Cow<'scope, BTreeMap<PreparedRelationEndpointKey, usize>>,
    pub(super) target_counts: Cow<'scope, BTreeMap<PreparedRelationEndpointKey, usize>>,
    pub(super) directed_pair_counts: Cow<'scope, BTreeMap<PreparedRelationPairKey, usize>>,
    pub(super) candidate_source_entities: BTreeSet<EntityReference>,
    pub(super) candidate_target_entities: BTreeSet<EntityReference>,
    pub(super) relation_slot_scans: usize,
    pub(super) entity_slot_scans: usize,
}

pub(super) fn visible_relation_counts<'scope>(
    context: &'scope InvariantExecutionContext<'_, '_>,
    contract: &LoweredCardinalityMinimumContract,
) -> VisibleRelationCountSnapshot<'scope> {
    if context.merged_plan().is_some() {
        return planned_visible_relation_counts(context, contract);
    }
    let state_view = context.state_view();
    let mut snapshot = VisibleRelationCountSnapshot::default();

    if state_view.version_id() == context.current_version_id() {
        let mut built_scans = None;
        let owned_index = context
            .is_leased()
            .then(|| CurrentVersionMinimumIndex::build(context));
        let index = owned_index.as_ref().unwrap_or_else(|| {
            context.current_version_minimum_index().get_or_init(|| {
                let index = CurrentVersionMinimumIndex::build(context);
                built_scans = Some((index.entity_slot_scans, index.relation_slot_scans));
                index
            })
        });
        if context.is_leased() {
            built_scans = Some((index.entity_slot_scans, index.relation_slot_scans));
        }
        if !context.checkpoint(0) {
            return snapshot;
        }
        if let Some((entity_scans, relation_scans)) = built_scans {
            snapshot.entity_slot_scans = entity_scans;
            snapshot.relation_slot_scans = relation_scans;
        }
        if let Some(relations) = index.relations.get(&contract.relation_kind_id) {
            for &(source, target) in relations {
                if !context.checkpoint(1) {
                    return snapshot;
                }
                if !context.claim_scratch(
                    (4 * std::mem::size_of::<EntityReference>() + 6 * std::mem::size_of::<usize>())
                        as u64,
                ) {
                    return snapshot;
                }
                record_existing_relation_endpoints(&mut snapshot, source, target);
            }
        }
        for (ordinal, kind_id) in contract
            .candidate_source_kinds
            .iter()
            .chain(&contract.candidate_target_kinds)
            .copied()
            .enumerate()
        {
            if !context.checkpoint((ordinal as u64).saturating_add(1)) {
                return snapshot;
            }
            if contract
                .candidate_source_kinds
                .iter()
                .chain(&contract.candidate_target_kinds)
                .take(ordinal)
                .any(|prior| *prior == kind_id)
            {
                continue;
            }
            if let Some(entities) = index.entities.get(&kind_id) {
                for &entity_id in entities {
                    if !context.checkpoint(1) {
                        return snapshot;
                    }
                    record_candidate_entity(
                        context,
                        contract,
                        &mut snapshot,
                        kind_id,
                        EntityReference::Existing(entity_id),
                    );
                }
            }
        }
        return snapshot;
    }

    for partition_id in state_view.state().partition_ids_iter() {
        if !context.checkpoint(1) {
            return snapshot;
        }
        let Some(partition) = state_view.state().get_current_partition(partition_id) else {
            continue;
        };
        collect_selected_root_relation_counts(context, contract, partition, &mut snapshot);
        collect_selected_root_candidate_entities(
            context,
            contract,
            partition_id,
            partition,
            &mut snapshot,
        );
    }

    snapshot
}

mod planned_counts;
use planned_counts::planned_visible_relation_counts;

fn collect_selected_root_relation_counts(
    context: &InvariantExecutionContext<'_, '_>,
    contract: &LoweredCardinalityMinimumContract,
    partition: crate::storage::residency::RelationalRegionHeadView<'_>,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    let state_view = context.state_view();
    for slot in partition.relation_arena().occupied_slots_iter() {
        if !context.checkpoint(1) {
            return;
        }
        context.metrics().count_relation_slot_scans(1);
        snapshot.relation_slot_scans += 1;
        let Some(metadata) =
            visible_relation_metadata(&state_view, &partition.relation_arena(), slot)
        else {
            continue;
        };
        if metadata.kind_id != contract.relation_kind_id {
            continue;
        }
        if !context.claim_scratch(
            (4 * std::mem::size_of::<EntityReference>() + 6 * std::mem::size_of::<usize>()) as u64,
        ) {
            return;
        }
        record_existing_relation_endpoints(
            snapshot,
            metadata.endpoints.source,
            metadata.endpoints.target,
        );
    }
}

fn collect_selected_root_candidate_entities(
    context: &InvariantExecutionContext<'_, '_>,
    contract: &LoweredCardinalityMinimumContract,
    partition_id: crate::identity::data::PartitionId,
    partition: crate::storage::residency::RelationalRegionHeadView<'_>,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    let state_view = context.state_view();
    for slot in partition.entity_arena().occupied_slots_iter() {
        if !context.checkpoint(1) {
            return;
        }
        context.metrics().count_entity_slot_scans(1);
        snapshot.entity_slot_scans += 1;
        let Some(metadata) =
            state_view.entity_metadata_at(&partition.entity_arena(), partition_id, slot)
        else {
            continue;
        };
        record_candidate_entity(
            context,
            contract,
            snapshot,
            metadata.kind_id,
            EntityReference::Existing(metadata.entity_id),
        );
    }
}

fn record_existing_relation_endpoints(
    snapshot: &mut VisibleRelationCountSnapshot,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
) {
    record_relation_references(
        snapshot,
        EntityReference::Existing(source),
        EntityReference::Existing(target),
    );
}

fn record_relation_references(
    snapshot: &mut VisibleRelationCountSnapshot,
    source: EntityReference,
    target: EntityReference,
) {
    *snapshot
        .source_counts
        .to_mut()
        .entry(PreparedRelationEndpointKey {
            entity_id: source.clone(),
        })
        .or_insert(0) += 1;
    *snapshot
        .target_counts
        .to_mut()
        .entry(PreparedRelationEndpointKey {
            entity_id: target.clone(),
        })
        .or_insert(0) += 1;
    *snapshot
        .directed_pair_counts
        .to_mut()
        .entry(PreparedRelationPairKey { source, target })
        .or_insert(0) += 1;
}

fn subtract_relation_references(
    snapshot: &mut VisibleRelationCountSnapshot,
    source: EntityReference,
    target: EntityReference,
) {
    decrement_count(
        snapshot.source_counts.to_mut(),
        PreparedRelationEndpointKey {
            entity_id: source.clone(),
        },
    );
    decrement_count(
        snapshot.target_counts.to_mut(),
        PreparedRelationEndpointKey {
            entity_id: target.clone(),
        },
    );
    decrement_count(
        snapshot.directed_pair_counts.to_mut(),
        PreparedRelationPairKey { source, target },
    );
}

fn decrement_count<Key: Ord>(counts: &mut BTreeMap<Key, usize>, key: Key) {
    if let Some(count) = counts.get_mut(&key) {
        *count -= 1;
        if *count == 0 {
            counts.remove(&key);
        }
    }
}

fn visible_relation_metadata<'state>(
    state_view: &InvariantStateView<'state>,
    arena: &crate::storage::residency::RelationalRecordHeadArenaView<
        'state,
        crate::storage::substrate::RelationRecordKind,
    >,
    slot: usize,
) -> Option<&'state VersionedRelationMetadata> {
    let metadata = arena.current_metadata_at(slot)?;
    state_view.metadata_is_visible(metadata).then_some(metadata)
}
