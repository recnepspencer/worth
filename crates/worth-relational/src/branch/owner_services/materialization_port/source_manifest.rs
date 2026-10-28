//! A probe-bound source manifest never interprets a cold region as missing output.
use super::*;

pub(super) fn source_generated_manifest(
    root: &dyn crate::storage::overlay::PartitionAccess,
    generated_entities: &[crate::identity::data::EntityId],
) -> Result<Vec<RelationalMaterializationRecord>, RelationalMaterializationError> {
    let generated = generated_entities.iter().copied().collect::<BTreeSet<_>>();
    if generated.is_empty() || generated.len() != generated_entities.len() {
        return Err(RelationalMaterializationError::SourceIsNotCompleteCreatePublication);
    }
    let mut unique = BTreeSet::new();
    let mut incident_relations = BTreeSet::new();
    for entity_id in &generated {
        unique.insert(live_record_manifest(
            root,
            &crate::transactions::data::RecordRef::Entity(*entity_id),
        )?);
        if let Some(partition) = root.get_partition(entity_id.partition_id) {
            let slot = entity_id.slot_index();
            for adjacency in [
                partition.adjacency.get(slot),
                partition.reverse_adjacency.get(slot),
            ]
            .into_iter()
            .flatten()
            {
                incident_relations.extend(adjacency.current_ids().iter().copied());
            }
        }
    }
    for relation_id in incident_relations {
        if let Some(record) = live_relation_manifest(root, relation_id)? {
            unique.insert(record);
        }
    }
    Ok(unique.into_iter().collect())
}

fn live_relation_manifest(
    root: &dyn crate::storage::overlay::PartitionAccess,
    relation_id: crate::identity::data::RelationId,
) -> Result<Option<RelationalMaterializationRecord>, RelationalMaterializationError> {
    let Some(slot) = root
        .get_partition(relation_id.partition_id)
        .and_then(|partition| partition.relation_arena.get(&relation_id))
        .filter(|slot| slot.is_live())
    else {
        return Ok(None);
    };
    let kind_id = slot
        .kind_id()
        .ok_or_else(|| missing_relation(relation_id))?;
    let endpoints = slot
        .extra()
        .endpoints
        .as_ref()
        .ok_or_else(|| missing_relation(relation_id))?;
    Ok(Some(RelationalMaterializationRecord::Relation {
        relation_id,
        kind_id,
        source: endpoints.source,
        target: endpoints.target,
    }))
}

fn live_record_manifest(
    root: &dyn crate::storage::overlay::PartitionAccess,
    record: &crate::transactions::data::RecordRef,
) -> Result<RelationalMaterializationRecord, RelationalMaterializationError> {
    match record {
        crate::transactions::data::RecordRef::Entity(entity_id) => {
            let slot = root
                .get_partition(entity_id.partition_id)
                .and_then(|partition| partition.entity_arena.get(entity_id));
            let kind_id = slot
                .filter(|slot| slot.is_live())
                .and_then(|slot| slot.kind_id())
                .ok_or_else(|| missing_entity(*entity_id))?;
            Ok(RelationalMaterializationRecord::Entity {
                entity_id: *entity_id,
                kind_id,
            })
        }
        crate::transactions::data::RecordRef::Relation(relation_id) => {
            let slot = root
                .get_partition(relation_id.partition_id)
                .and_then(|partition| partition.relation_arena.get(relation_id))
                .filter(|slot| slot.is_live())
                .ok_or_else(|| missing_relation(*relation_id))?;
            let kind_id = slot
                .kind_id()
                .ok_or_else(|| missing_relation(*relation_id))?;
            let endpoints = slot
                .extra()
                .endpoints
                .as_ref()
                .ok_or_else(|| missing_relation(*relation_id))?;
            Ok(RelationalMaterializationRecord::Relation {
                relation_id: *relation_id,
                kind_id,
                source: endpoints.source,
                target: endpoints.target,
            })
        }
    }
}
