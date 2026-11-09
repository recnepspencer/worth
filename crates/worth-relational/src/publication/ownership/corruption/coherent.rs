//! Mutate both head columns and their ordinary current adjacency, preserving history.
use super::*;
use crate::identity::data::EntityId;
use crate::{
    durability::data::*, identity::data::*, schema::data::CurrentPublicationStorage,
    storage::data::RecordLifecycleState,
};
use std::collections::BTreeSet;
mod claim;

pub(super) fn alter(
    partition: &mut PartitionCheckpointImage,
    publication: CommitId,
    change: u8,
    storage: &CurrentPublicationStorage,
) -> bool {
    if change == 9 {
        return claim::repoint(partition, publication, storage);
    }
    let mut entities = BTreeSet::new();
    let mut changed = false;
    for index in 0..partition.entity_arena.extra.len() {
        let kind = partition.entity_arena.kind_ids[index];
        let original = partition.entity_arena.extra[index]
            .authoritative_aspect_state
            .as_ref();
        let owns = |aspect| payload(original, aspect).is_some_and(|(p, _)| p == publication.0);
        let roster = change == 3 || change == 5;
        let position = change == 4 || change == 5;
        let mut remove = roster
            && ((kind == Some(storage.publication_kind)
                && owns(crate::schema::current_publication_ownership::PUBLICATION))
                || (kind == Some(storage.edge_kind)
                    && owns(crate::schema::current_publication_ownership::EDGE)))
            || position
                && kind == Some(storage.position_kind)
                && owns(crate::schema::current_publication_ownership::POSITION);
        if roster && kind == Some(storage.catalog_kind) {
            let remove_route = |state: &mut Option<PortableRecordAspectState>| {
                rewrite(
                    state,
                    crate::schema::current_publication_ownership::CATALOG,
                    |text| {
                        let mut value: record::Catalog = serde_json::from_str(text).ok()?;
                        let before = value.routes.len();
                        value.routes.retain(|r| r.publication != publication);
                        if before == value.routes.len() {
                            return None;
                        }
                        serde_json::to_string(&value).ok()
                    },
                )
            };
            let removed_route =
                remove_route(&mut partition.entity_arena.extra[index].authoritative_aspect_state);
            changed |= removed_route;
            for meta in &mut partition.entity_arena.metadata_history[index] {
                remove_route(&mut meta.authoritative_aspect_state);
            }
            remove = removed_route
                && payload(
                    partition.entity_arena.extra[index]
                        .authoritative_aspect_state
                        .as_ref(),
                    crate::schema::current_publication_ownership::CATALOG,
                )
                .and_then(|(_, text)| serde_json::from_str::<record::Catalog>(text).ok())
                .is_some_and(|c| c.routes.is_empty());
        }
        if remove {
            let slot = partition
                .entity_arena
                .slots
                .get(index)
                .copied()
                .unwrap_or(index as u64);
            entities.insert(EntityId::new(
                partition.partition_id,
                slot,
                partition.entity_arena.generations[index],
            ));
            delete(&mut partition.entity_arena, index, slot);
            if let Some(meta) = partition.entity_arena.metadata_history[index].last_mut() {
                meta.retired_at = Some(meta.effective_at);
            }
            changed = true;
        }
        if matches!(change, 6..=8) {
            let aspect = if change != 7 {
                crate::schema::current_publication_ownership::CATALOG
            } else {
                crate::schema::current_publication_ownership::POSITION
            };
            let mutate = |state: &mut Option<PortableRecordAspectState>| {
                rewrite(state, aspect, |text| {
                    if change == 8 {
                        let value: record::Catalog = serde_json::from_str(text).ok()?;
                        return value
                            .routes
                            .iter()
                            .any(|r| r.publication == publication)
                            .then(|| "{".to_owned());
                    }
                    if change == 6 {
                        let mut value: record::Catalog = serde_json::from_str(text).ok()?;
                        let route = value
                            .routes
                            .iter_mut()
                            .find(|r| r.publication == publication)?;
                        route.digest[0] ^= 1;
                        serde_json::to_string(&value).ok()
                    } else {
                        let mut value: crate::history::data::current_route_presence::RoutePresence =
                            serde_json::from_str(text).ok()?;
                        if value.publication != publication {
                            return None;
                        }
                        value.digest[0] ^= 1;
                        serde_json::to_string(&value).ok()
                    }
                })
            };
            changed |= mutate(&mut partition.entity_arena.extra[index].authoritative_aspect_state);
            for meta in &mut partition.entity_arena.metadata_history[index] {
                mutate(&mut meta.authoritative_aspect_state);
            }
        }
    }
    let mut relations = BTreeSet::new();
    for index in 0..partition.relation_arena.extra.len() {
        let kind = partition.relation_arena.kind_ids[index];
        let extra = &partition.relation_arena.extra[index];
        let p = payload(
            extra.authoritative_aspect_state.as_ref(),
            crate::schema::current_publication_ownership::LINK,
        )
        .map(|(p, _)| p);
        let position = kind == Some(storage.position_link);
        let roster = kind.is_some_and(|k| {
            [
                storage.member_link,
                storage.edge_link,
                storage.publication_link,
            ]
            .contains(&k)
        });
        let remove = change == 10 && kind == Some(storage.member_link) && p == Some(publication.0)
            || p == Some(publication.0)
                && ((position && matches!(change, 4 | 5)) || (roster && matches!(change, 3 | 5)))
            || extra
                .endpoints
                .as_ref()
                .is_some_and(|e| entities.contains(&e.source) || entities.contains(&e.target));
        if remove {
            let slot = partition
                .relation_arena
                .slots
                .get(index)
                .copied()
                .unwrap_or(index as u64);
            relations.insert(RelationId::new(
                partition.partition_id,
                slot,
                partition.relation_arena.generations[index],
            ));
            delete(&mut partition.relation_arena, index, slot);
            if let Some(meta) = partition.relation_arena.metadata_history[index].last_mut() {
                meta.retired_at = Some(meta.effective_at);
            }
            changed = true;
        }
    }
    for adjacency in partition
        .adjacency
        .iter_mut()
        .chain(&mut partition.reverse_adjacency)
    {
        adjacency.relations.retain(|id| !relations.contains(id));
    }
    changed
}
fn delete<K: RecordArenaCheckpointKind>(
    arena: &mut RecordArenaCheckpointImage<K>,
    index: usize,
    slot: u64,
) {
    arena.lifecycle[index] = RecordLifecycleState::DeletedRetained;
    arena.retired_at[index] = Some(arena.created_at[index]);
    let word = slot / 64;
    let bit = !(1 << (slot % 64));
    if let Some(value) = arena.live_bitset.words.get_mut(word as usize) {
        *value &= bit;
    }
    for (key, value) in &mut arena.live_bitset.sparse_words {
        if *key == word {
            *value &= bit;
        }
    }
    arena.live_bitset.sparse_words.retain(|(_, v)| *v != 0);
}
fn payload<'a>(
    state: Option<&'a PortableRecordAspectState>,
    aspect: &str,
) -> Option<(u64, &'a str)> {
    let state = state?;
    let entry = state
        .entries()
        .iter()
        .find(|e| e.basis().key().as_str() == aspect)?;
    let ContractValidationInput::Struct(fields) = entry.value() else {
        return None;
    };
    let AspectValue::UInt64(publication) = fields.get(&FieldKey::new("publication")?)? else {
        return None;
    };
    let Some(text) = (fields.get(&FieldKey::new("payload")?)?).raw_text() else {
        return None;
    };
    Some((*publication, text))
}
fn rewrite(
    state: &mut Option<PortableRecordAspectState>,
    aspect: &str,
    mut change: impl FnMut(&str) -> Option<String>,
) -> bool {
    let Some(original) = state else {
        return false;
    };
    let mut changed = false;
    let entries = original
        .entries()
        .iter()
        .map(|entry| {
            if entry.basis().key().as_str() != aspect {
                return entry.clone();
            }
            let ContractValidationInput::Struct(fields) = entry.value() else {
                return entry.clone();
            };
            let payload = FieldKey::new("payload").unwrap();
            let Some(text) = (fields.get(&payload))
                .as_ref()
                .and_then(|value| value.raw_text())
            else {
                return entry.clone();
            };
            let Some(text) = change(text) else {
                return entry.clone();
            };
            changed = true;
            let value = StructAspectValue::new(fields.fields().map(|(key, value)| {
                (
                    key.clone(),
                    if key == &payload {
                        AspectValue::String(text.clone().into())
                    } else {
                        value.clone()
                    },
                )
            }))
            .unwrap();
            PortableRecordAspectStateEntry::new(entry.basis().clone(), value.into())
        })
        .collect::<Vec<_>>();
    if changed {
        *state = Some(PortableRecordAspectState::new(entries));
    }
    changed
}
