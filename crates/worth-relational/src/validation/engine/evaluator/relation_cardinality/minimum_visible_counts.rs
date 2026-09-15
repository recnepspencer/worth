use std::collections::{BTreeMap, BTreeSet};

use crate::schema::data::LoweredCardinalityMinimumContract;
use crate::storage::substrate::{HistoricalMetadata, RelationArena, VersionedRelationMetadata};
use crate::transactions::data::{CreatedEntityRef, EntityReference};

use super::super::super::context::InvariantExecutionContext;
use super::super::super::state_view::InvariantStateView;
use super::super::common::contract_candidate_kind_matches;

#[derive(Default)]
pub(super) struct VisibleRelationCountSnapshot {
    pub(super) source_counts: BTreeMap<EntityReference, usize>,
    pub(super) target_counts: BTreeMap<EntityReference, usize>,
    pub(super) directed_pair_counts: BTreeMap<(EntityReference, EntityReference), usize>,
    pub(super) candidate_source_entities: BTreeSet<EntityReference>,
    pub(super) candidate_target_entities: BTreeSet<EntityReference>,
    pub(super) relation_slot_scans: usize,
    pub(super) entity_slot_scans: usize,
}

pub(super) fn visible_relation_counts(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
) -> VisibleRelationCountSnapshot {
    let state_view = context.state_view();
    let mut snapshot = VisibleRelationCountSnapshot::default();

    for partition_id in state_view.state().partition_ids() {
        let Some(partition) = state_view.state().get_partition(partition_id) else {
            continue;
        };
        if state_view.version_id() == context.current_version_id() {
            collect_current_version_relation_counts(context, contract, partition, &mut snapshot);
            collect_current_version_candidate_entities(
                context,
                contract,
                partition_id,
                partition,
                &mut snapshot,
            );
        } else {
            collect_historical_relation_counts(context, contract, partition, &mut snapshot);
            collect_historical_candidate_entities(
                context,
                contract,
                partition_id,
                partition,
                &mut snapshot,
            );
        }
    }

    collect_planned_counts(context, contract, &mut snapshot);
    snapshot
}

fn collect_current_version_relation_counts(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
    partition: &crate::storage::overlay::PartitionState,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    for slot in partition.relation_arena.live_bitset.iter_set_slots() {
        context.metrics().count_relation_slot_scans(1);
        snapshot.relation_slot_scans += 1;
        let Some(slot_view) = partition.relation_arena.get_slot(slot) else {
            continue;
        };
        let Some(kind_id) = slot_view.kind_id() else {
            continue;
        };
        if kind_id != contract.relation_kind_id {
            continue;
        }
        let Some(endpoints) = slot_view.extra().endpoints.as_ref() else {
            continue;
        };
        record_existing_relation_endpoints(snapshot, endpoints.source, endpoints.target);
    }
}

fn collect_current_version_candidate_entities(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
    partition_id: crate::identity::data::PartitionId,
    partition: &crate::storage::overlay::PartitionState,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    for slot in partition.entity_arena.live_bitset.iter_set_slots() {
        context.metrics().count_entity_slot_scans(1);
        snapshot.entity_slot_scans += 1;
        let Some(slot_view) = partition.entity_arena.get_slot(slot) else {
            continue;
        };
        let Some(kind_id) = slot_view.kind_id() else {
            continue;
        };
        let entity_id =
            crate::identity::data::EntityId::new(partition_id, slot as u64, slot_view.generation());
        record_candidate_entity(
            contract,
            snapshot,
            kind_id,
            EntityReference::Existing(entity_id),
        );
    }
}

fn collect_historical_relation_counts(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
    partition: &crate::storage::overlay::PartitionState,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    let state_view = context.state_view();
    for slot in partition.relation_arena.occupied_slots() {
        context.metrics().count_relation_slot_scans(1);
        snapshot.relation_slot_scans += 1;
        let Some(metadata) =
            visible_relation_metadata(&state_view, &partition.relation_arena, slot)
        else {
            continue;
        };
        if metadata.kind_id != contract.relation_kind_id {
            continue;
        }
        record_existing_relation_endpoints(
            snapshot,
            metadata.endpoints.source,
            metadata.endpoints.target,
        );
    }
}

fn collect_historical_candidate_entities(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
    partition_id: crate::identity::data::PartitionId,
    partition: &crate::storage::overlay::PartitionState,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    let state_view = context.state_view();
    for slot in partition.entity_arena.occupied_slots() {
        context.metrics().count_entity_slot_scans(1);
        snapshot.entity_slot_scans += 1;
        let Some(metadata) =
            state_view.entity_metadata_at(&partition.entity_arena, partition_id, slot)
        else {
            continue;
        };
        record_candidate_entity(
            contract,
            snapshot,
            metadata.kind_id,
            EntityReference::Existing(metadata.entity_id),
        );
    }
}

fn collect_planned_counts(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    let Some(merged_plan) = context.merged_plan() else {
        return;
    };
    for intent in &merged_plan.merged_intents {
        match intent {
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::Entity(spec),
            ) => {
                let entity = EntityReference::Created(CreatedEntityRef {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    client_key: spec.client_key.clone(),
                });
                record_candidate_entity(contract, snapshot, spec.kind_id, entity);
            }
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::BulkEntities(spec),
            ) => {
                for client_key in &spec.client_keys {
                    let entity = EntityReference::Created(CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                    });
                    record_candidate_entity(contract, snapshot, spec.kind_id, entity);
                }
            }
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::Relation(spec),
            ) => {
                if spec.kind_id == contract.relation_kind_id {
                    record_relation_references(snapshot, spec.source.clone(), spec.target.clone());
                }
            }
            crate::transactions::data::MutationIntent::Create(
                crate::transactions::data::CreateIntent::BulkRelations(spec),
            ) => {
                if spec.kind_id == contract.relation_kind_id {
                    for (source, target) in &spec.endpoints {
                        record_relation_references(snapshot, source.clone(), target.clone());
                    }
                }
            }
            crate::transactions::data::MutationIntent::Materialization(
                crate::transactions::data::MaterializationMutationIntent::RematerializeRelation(
                    spec,
                ),
            ) => {
                if spec.kind_id == contract.relation_kind_id {
                    record_relation_references(
                        snapshot,
                        EntityReference::Existing(spec.source),
                        EntityReference::Existing(spec.target),
                    );
                }
            }
            _ => {}
        }
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
    *snapshot.source_counts.entry(source.clone()).or_insert(0) += 1;
    *snapshot.target_counts.entry(target.clone()).or_insert(0) += 1;
    *snapshot
        .directed_pair_counts
        .entry((source, target))
        .or_insert(0) += 1;
}

fn record_candidate_entity(
    contract: &LoweredCardinalityMinimumContract,
    snapshot: &mut VisibleRelationCountSnapshot,
    kind_id: crate::identity::data::KindId,
    entity: EntityReference,
) {
    if contract_candidate_kind_matches(kind_id, &contract.candidate_source_kinds) {
        snapshot.candidate_source_entities.insert(entity.clone());
    }
    if contract_candidate_kind_matches(kind_id, &contract.candidate_target_kinds) {
        snapshot.candidate_target_entities.insert(entity);
    }
}

fn visible_relation_metadata<'state>(
    state_view: &InvariantStateView<'state>,
    arena: &'state RelationArena,
    slot: usize,
) -> Option<&'state VersionedRelationMetadata> {
    let history = arena.metadata_history_at(slot)?;
    let end = history.partition_point(|entry| entry.effective_at() <= state_view.version_id());
    history[..end].iter().rev().find(|entry| {
        entry.effective_at() <= state_view.version_id()
            && entry
                .retired_at()
                .is_none_or(|retired| state_view.version_id() < retired)
    })
}
