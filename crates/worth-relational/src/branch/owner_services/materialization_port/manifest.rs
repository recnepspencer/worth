//! Authored payload closure; Native-owned identity metadata remains materialized.
//! O(C log C + (E+R) log(C+E+R)) work and O(C+E+R) scratch for schema contracts
//! C, selected entities E and their incident relations R; unrelated roots are
//! never traversed. The actual selected root supplies ownership declarations.
use super::*;

pub(super) fn source_generated_manifest(
    basis: &AdmittedRelationalBranchBasis,
    generated_entities: &[crate::identity::data::EntityId],
) -> Result<Vec<RelationalMaterializationRecord>, RelationalMaterializationError> {
    let root = &basis.inner.root;
    let native_metadata = root
        .schema_authority()
        .registry()
        .relation_kinds
        .values()
        .filter_map(|kind| kind.relation_integrity.aggregate_identity.as_ref())
        .flat_map(|contract| contract.metadata_kinds())
        .collect::<BTreeSet<_>>();
    let generated = generated_entities.iter().copied().collect::<BTreeSet<_>>();
    if generated.is_empty() || generated.len() != generated_entities.len() {
        return Err(RelationalMaterializationError::SourceIsNotCompleteCreatePublication);
    }
    let mut unique = BTreeSet::new();
    let mut selected_native_metadata = BTreeSet::new();
    let mut incident_relations = BTreeSet::new();
    for entity_id in &generated {
        let record = live_record_manifest(
            root,
            &crate::transactions::data::RecordRef::Entity(*entity_id),
        )?;
        let RelationalMaterializationRecord::Entity { kind_id, .. } = record else {
            unreachable!("entity manifest requested");
        };
        if native_metadata.contains(&kind_id) {
            selected_native_metadata.insert(*entity_id);
        }
        unique.insert(record);
        if let Some(partition) = root.partition_state(entity_id.partition_id) {
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
            let RelationalMaterializationRecord::Relation {
                kind_id,
                source,
                target,
                ..
            } = record
            else {
                unreachable!("relation manifest requested");
            };
            // Authored payload suspension leaves Native metadata materialized.
            // A Native storage owner explicitly selecting its private entity
            // still owns the complete incident metadata closure it must restore.
            if !native_metadata.contains(&kind_id)
                || selected_native_metadata.contains(&source)
                || selected_native_metadata.contains(&target)
            {
                unique.insert(record);
            }
        }
    }
    Ok(unique.into_iter().collect())
}

fn live_relation_manifest(
    root: &crate::branch::RelationalBranchRoot,
    relation_id: crate::identity::data::RelationId,
) -> Result<Option<RelationalMaterializationRecord>, RelationalMaterializationError> {
    let Some(slot) = root
        .partition_state(relation_id.partition_id)
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

pub(super) fn live_record_manifest(
    root: &crate::branch::RelationalBranchRoot,
    record: &crate::transactions::data::RecordRef,
) -> Result<RelationalMaterializationRecord, RelationalMaterializationError> {
    match record {
        crate::transactions::data::RecordRef::Entity(entity_id) => {
            let slot = root
                .partition_state(entity_id.partition_id)
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
                .partition_state(relation_id.partition_id)
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
