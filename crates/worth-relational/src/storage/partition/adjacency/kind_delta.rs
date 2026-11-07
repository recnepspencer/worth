//! Persistent kind buckets expose only changed keys to region publication.
use super::{AdjacencyEntries, AdjacencySet};
use crate::identity::data::{KindId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::AdjacencyKindBasis;

impl AdjacencySet {
    pub(crate) fn remove_historical_kind(&mut self, kind: KindId, relation: RelationId) {
        if let Some(relations) = self
            .entries_mut()
            .historical_by_kind
            .as_mut()
            .and_then(|kinds| kinds.get_mut(&kind))
        {
            relations.remove(&relation);
        }
    }

    pub(crate) fn contains_historical_kind(&self, kind: KindId, relation: RelationId) -> bool {
        self.entries()
            .historical_by_kind
            .as_ref()
            .and_then(|kinds| kinds.get(&kind))
            .is_some_and(|relations| relations.contains_key(&relation))
    }

    pub(crate) fn visit_changed_kind_memberships(
        &self,
        previous: Option<&Self>,
        basis: AdjacencyKindBasis,
        checkpoint: &mut dyn FnMut(u64, u64) -> Result<(), Cause>,
        visit: &mut dyn FnMut(KindId, RelationId, bool) -> Result<(), Cause>,
    ) -> Result<(), Cause> {
        let select = |entries: &AdjacencyEntries| match basis {
            AdjacencyKindBasis::Current => entries.current_by_kind.clone(),
            AdjacencyKindBasis::Historical => entries.historical_by_kind.clone(),
        };
        let current = select(self.entries()).unwrap_or_default();
        let previous = previous
            .and_then(|old| select(old.entries()))
            .unwrap_or_default();
        // Shared subtrees are skipped by storage allocation identity, including
        // retained history. A changed bucket visits only its changed members.
        checkpoint(0, 0)?;
        for kind in current.try_changed_keys_since(&previous, checkpoint)? {
            checkpoint(
                (current.navigation_work_bound() + previous.navigation_work_bound()) as u64,
                512,
            )?;
            let old = previous.get(&kind).cloned().unwrap_or_default();
            let new = current.get(&kind).cloned().unwrap_or_default();
            for relation in new.try_changed_keys_since(&old, checkpoint)? {
                checkpoint(
                    (new.navigation_work_bound() + old.navigation_work_bound()) as u64,
                    512,
                )?;
                visit(kind, relation, new.contains_key(&relation))?;
            }
        }
        Ok(())
    }
}
