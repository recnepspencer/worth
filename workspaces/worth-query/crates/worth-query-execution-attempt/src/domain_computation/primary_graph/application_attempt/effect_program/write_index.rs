use std::collections::BTreeMap;

use worth_relational::facade::identity::EntityId;

/// Builder-local positions of authored entity-field effects. Effects only append
/// or change in place before finish, so a position stays valid throughout authoring.
#[derive(Default)]
pub(super) struct FieldWriteIndex {
    by_entity: BTreeMap<EntityId, BTreeMap<String, usize>>,
}

impl FieldWriteIndex {
    pub(super) fn position(&self, entity: &str, entity_id: EntityId) -> Option<usize> {
        self.by_entity.get(&entity_id)?.get(entity).copied()
    }

    pub(super) fn remember(&mut self, entity: &str, entity_id: EntityId, position: usize) {
        let previous = self
            .by_entity
            .entry(entity_id)
            .or_default()
            .insert(entity.to_owned(), position);
        debug_assert!(previous.is_none(), "one field effect per entity identity");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::identity::PartitionId;

    #[test]
    fn positions_follow_distinct_entities_and_preserve_first_effect_order() {
        let mut index = FieldWriteIndex::default();
        for slot in 0..10_000 {
            let id = EntityId::new(PartitionId::new(1), slot, 1);
            assert_eq!(index.position("Account", id), None);
            index.remember("Account", id, slot as usize + 3);
        }
        for slot in 0..10_000 {
            let id = EntityId::new(PartitionId::new(1), slot, 1);
            assert_eq!(index.position("Account", id), Some(slot as usize + 3));
            assert_eq!(index.position("Principal", id), None);
        }
    }
}
