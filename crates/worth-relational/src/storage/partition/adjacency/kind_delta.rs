//! Persistent kind buckets expose only changed keys to region publication.
use super::*;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::partition::AdjacencyKindBasis;

impl AdjacencySet {
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
        let previous = previous.and_then(|old| select(old.entries())).unwrap_or_default();
        // Shared subtrees are skipped by storage allocation identity, including
        // retained history. A changed bucket visits only its changed members.
        checkpoint(0, 65_536)?;
        for kind in current.changed_keys_since(&previous) {
            checkpoint((current.navigation_work_bound() + previous.navigation_work_bound()) as u64, 512)?;
            let old = previous.get(&kind).cloned().unwrap_or_default();
            let new = current.get(&kind).cloned().unwrap_or_default();
            for relation in new.changed_keys_since(&old) {
                checkpoint((new.navigation_work_bound() + old.navigation_work_bound()) as u64, 512)?;
                visit(kind, relation, new.contains_key(&relation))?;
            }
        }
        Ok(())
    }
}
