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

#[cfg(test)]
mod tests {
    use super::super::{DecisionReadOutcome, DecisionReads};
    use super::*;
    use crate::domain_computation::primary_graph::application_attempt::ComputationRead;
    use std::collections::BTreeSet;
    use worth_foundational::facade::FieldKey;
    use worth_relational::facade::{
        identity::PartitionId, transactions::planned_single_field_locator,
    };

    fn presence(slot: u64, aspect: &str, field: &str) -> Key {
        Key::AspectPresence {
            entity: "Draft".to_owned(),
            entity_id: EntityId::new(PartitionId::main(), slot, 1),
            locator: planned_single_field_locator(
                AspectKey::new(aspect).unwrap(),
                FieldKey::new(field).unwrap(),
            ),
        }
    }

    #[test]
    fn optional_aspect_initialization_coalesces_presence_without_dropping_attribution() {
        let mut reads = DecisionReads::default();
        let first = presence(1, "facts", "note");
        let second = presence(1, "facts", "score");
        reads.record(first.clone(), DecisionReadOutcome::Observed);
        reads.reading = Some(ComputationRead::Membership);
        reads.record(second.clone(), DecisionReadOutcome::Observed);
        reads.reading = None;
        assert!(reads.computation.contains_key(&first));
        assert!(!reads.computation.contains_key(&second));
        let other_aspect = presence(1, "other", "note");
        let other_entity = presence(2, "facts", "note");
        reads.record(other_aspect.clone(), DecisionReadOutcome::Observed);
        reads.record(other_entity.clone(), DecisionReadOutcome::Observed);
        let (keys, _) = reads.into_expected().ok().unwrap();
        assert_eq!(
            keys.iter().cloned().collect::<BTreeSet<_>>(),
            BTreeSet::from([first, other_aspect, other_entity])
        );
    }
}
