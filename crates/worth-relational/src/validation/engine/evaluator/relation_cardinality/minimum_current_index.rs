use std::collections::BTreeMap;

use super::super::super::context::InvariantExecutionContext;
use crate::identity::data::{EntityId, KindId};

/// One current-version scan shared by serial minimum registrations. Leased
/// execution builds a local copy so a stopped scan cannot enter the cache.
pub(crate) struct CurrentVersionMinimumIndex {
    pub(super) entities: BTreeMap<KindId, Vec<EntityId>>,
    pub(super) relations: BTreeMap<KindId, Vec<(EntityId, EntityId)>>,
    pub(super) entity_slot_scans: usize,
    pub(super) relation_slot_scans: usize,
}

impl CurrentVersionMinimumIndex {
    pub(super) fn build(context: &InvariantExecutionContext<'_, '_>) -> Self {
        let mut index = Self {
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
            entity_slot_scans: 0,
            relation_slot_scans: 0,
        };
        let state = context.state_view();
        for partition_id in state.state().partition_ids_iter() {
            if !context.checkpoint(1) {
                return index;
            }
            let Some(partition) = state.state().get_partition(partition_id) else {
                continue;
            };
            for slot in partition.entity_arena.live_bitset.set_slots() {
                if !context.checkpoint(1) {
                    return index;
                }
                index.entity_slot_scans += 1;
                let Some(view) = partition.entity_arena.get_slot(slot) else {
                    continue;
                };
                let Some(kind_id) = view.kind_id() else {
                    continue;
                };
                if !context.claim_scratch(
                    (std::mem::size_of::<EntityId>() + 3 * std::mem::size_of::<usize>()) as u64,
                ) {
                    return index;
                }
                index
                    .entities
                    .entry(kind_id)
                    .or_default()
                    .push(EntityId::new(partition_id, slot as u64, view.generation()));
            }
            for slot in partition.relation_arena.live_bitset.set_slots() {
                if !context.checkpoint(1) {
                    return index;
                }
                index.relation_slot_scans += 1;
                let Some(view) = partition.relation_arena.get_slot(slot) else {
                    continue;
                };
                let (Some(kind_id), Some(endpoints)) =
                    (view.kind_id(), view.extra().endpoints.as_ref())
                else {
                    continue;
                };
                if !context.claim_scratch(
                    (2 * std::mem::size_of::<EntityId>() + 3 * std::mem::size_of::<usize>()) as u64,
                ) {
                    return index;
                }
                index
                    .relations
                    .entry(kind_id)
                    .or_default()
                    .push((endpoints.source, endpoints.target));
            }
        }
        context
            .metrics()
            .count_entity_slot_scans(index.entity_slot_scans);
        context
            .metrics()
            .count_relation_slot_scans(index.relation_slot_scans);
        index
    }
}
