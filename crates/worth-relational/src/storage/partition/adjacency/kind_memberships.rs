//! Canonical kind-incidence traversal and checkpoint projection.
use super::super::AdjacencyKindBasis;
use super::{AdjacencySet, KindId, RelationId};

impl AdjacencySet {
    /// Borrow canonical kind incidences without constructing checkpoint Vecs.
    pub(crate) fn kind_memberships(
        &self,
        basis: AdjacencyKindBasis,
    ) -> impl Iterator<Item = (KindId, RelationId)> + '_ {
        let entries = self.entries();
        let selected = match basis {
            AdjacencyKindBasis::Current => &entries.current_by_kind,
            AdjacencyKindBasis::Historical => &entries.historical_by_kind,
        };
        selected
            .iter()
            .flat_map(|map| map.iter())
            .flat_map(|(&kind, ids)| ids.keys().map(move |&id| (kind, id)))
    }

    /// Borrow memberships one at a time; the receiver owns admission per item.
    pub(crate) fn visit_kind_memberships(
        &self,
        basis: AdjacencyKindBasis,
        visit: &mut dyn FnMut(
            KindId,
            Option<RelationId>,
        ) -> Result<
            (),
            crate::storage::data::RelationalStorageUnavailableCause,
        >,
    ) -> Result<(), crate::storage::data::RelationalStorageUnavailableCause> {
        let entries = self.entries();
        let selected = if matches!(basis, AdjacencyKindBasis::Historical) {
            &entries.historical_by_kind
        } else {
            &entries.current_by_kind
        };
        if let Some(kinds) = selected {
            for (&kind, ids) in kinds {
                visit(kind, None)?;
                for &relation in ids.keys() {
                    visit(kind, Some(relation))?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn checkpoint_kind_memberships(
        &self,
        basis: AdjacencyKindBasis,
    ) -> Vec<(KindId, Vec<RelationId>)> {
        let entries = self.entries();
        let selected = if matches!(basis, AdjacencyKindBasis::Historical) {
            &entries.historical_by_kind
        } else {
            &entries.current_by_kind
        };
        selected
            .as_ref()
            .map(|kinds| {
                kinds
                    .iter()
                    .map(|(&kind, ids)| (kind, ids.keys().copied().collect()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
