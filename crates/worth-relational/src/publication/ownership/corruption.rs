//! Closed corruption of current facts; the ordinary reader must reject the result.
use super::record;
mod coherent;
use crate::{durability::data::PartitionCheckpointImage, history::data::CommitId};
use worth_foundational::facade::*;

pub(crate) fn alter(
    partition: &mut PartitionCheckpointImage,
    publication: CommitId,
    change: u8,
    storage: &crate::schema::data::CurrentPublicationStorage,
) -> bool {
    if change >= 3 {
        return coherent::alter(partition, publication, change, storage);
    }
    let mut changed = false;
    for extra in &mut partition.entity_arena.extra {
        changed |= state(&mut extra.authoritative_aspect_state, publication, change);
    }
    for history in &mut partition.entity_arena.metadata_history {
        for meta in history {
            changed |= state(&mut meta.authoritative_aspect_state, publication, change);
        }
    }
    changed
}
fn state(state: &mut Option<PortableRecordAspectState>, publication: CommitId, change: u8) -> bool {
    let Some(original) = state.as_ref() else {
        return false;
    };
    let mut changed = false;
    let entries = original
        .entries()
        .iter()
        .map(|entry| {
            if entry.basis().key().as_str() != crate::schema::current_publication_ownership::CATALOG
            {
                return entry.clone();
            }
            let ContractValidationInput::Struct(fields) = entry.value() else {
                return entry.clone();
            };
            let payload = FieldKey::new("payload").expect("fixed payload field");
            let Some(AspectValue::String(InternedString::Raw(text))) = fields.get(&payload) else {
                return entry.clone();
            };
            let Ok(mut catalog) = serde_json::from_str::<record::Catalog>(text) else {
                return entry.clone();
            };
            let Some(index) = catalog
                .routes
                .iter()
                .position(|route| route.publication == publication)
            else {
                return entry.clone();
            };
            match change {
                0 => {
                    catalog.routes.remove(index);
                }
                1 => {
                    let mut extra = catalog.routes[index].clone();
                    extra.meaning.family.push_str(".foreign");
                    catalog.routes.push(extra);
                }
                2 => {
                    catalog.routes[index].meaning.source_scope.generation.0 += 1;
                }
                _ => unreachable!("closed corruption choice"),
            }
            let text = serde_json::to_string(&catalog).expect("the real catalog grammar encodes");
            changed = true;
            let replacement = StructAspectValue::new(fields.fields().map(|(key, value)| {
                (
                    key.clone(),
                    if key == &payload {
                        AspectValue::String(text.clone().into())
                    } else {
                        value.clone()
                    },
                )
            }))
            .expect("the original field keys remain unique");
            PortableRecordAspectStateEntry::new(entry.basis().clone(), replacement.into())
        })
        .collect::<Vec<_>>();
    if changed {
        *state = Some(PortableRecordAspectState::new(entries));
    }
    changed
}
