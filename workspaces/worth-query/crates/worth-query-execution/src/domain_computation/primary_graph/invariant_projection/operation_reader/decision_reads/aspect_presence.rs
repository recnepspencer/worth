//! One aspect-presence dependency retains its first admitted field binding.
use std::collections::BTreeMap;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey as Key;

#[derive(Default)]
pub(super) struct PresenceKeys {
    keys: BTreeMap<(String, EntityId, AspectKey), Key>,
}

impl PresenceKeys {
    pub(super) fn canonical_key(&mut self, key: Key) -> Key {
        let Key::AspectPresence {
            entity,
            entity_id,
            locator,
        } = &key
        else {
            return key;
        };
        // All keys belong to one sealed projection observation. Reusing the
        // first binding preserves graph-read admission and lets every reader
        // attribute the same dependency without duplicate provider locators.
        self.keys
            .entry((
                entity.clone(),
                *entity_id,
                locator.aspect().aspect_key().clone(),
            ))
            .or_insert(key)
            .clone()
    }
}
