//! Join rows and the complete persistent leg state that maintains them.
use super::join_leg_state::JoinLegsState;
use super::{DerivedIndexEntryMap, RelationJoinEntry, RelationJoinKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationJoinIndex {
    entries: DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>,
    legs: Option<Arc<JoinLegsState>>,
}
impl RelationJoinIndex {
    pub(crate) fn maintained(
        entries: DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>,
        legs: JoinLegsState,
    ) -> Self {
        Self {
            entries,
            legs: Some(Arc::new(legs)),
        }
    }
    pub(crate) fn legs(&self) -> Option<&Arc<JoinLegsState>> {
        self.legs.as_ref()
    }
    pub(crate) fn install_legs(&mut self, legs: JoinLegsState) {
        self.legs = Some(Arc::new(legs));
    }
    /// cost-lane: cold restore/import. Validate both carried membership axes.
    pub(crate) fn consistent_legs(&self) -> bool {
        let Some(legs) = &self.legs else {
            return false;
        };
        legs.left.consistent()
            && legs.right.consistent()
            && self.entries.iter().all(|(key, rows)| {
                !rows.is_empty()
                    && rows.iter().all(|row| {
                        legs.left
                            .minimum(row.shared_entity_id(), key.left_entity_id())
                            == Some(row.left_relation_id())
                            && legs
                                .right
                                .minimum(row.shared_entity_id(), key.right_entity_id())
                                == Some(row.right_relation_id())
                    })
            })
            && legs.left.iter().all(|(_, (shared, left))| {
                legs.right.members(shared).all(|(right, _)| {
                    self.entries
                        .get(&RelationJoinKey::new(left, right))
                        .is_some_and(|rows| {
                            rows.binary_search_by(|row| row.shared_entity_id().cmp(&shared))
                                .is_ok()
                        })
                })
            })
    }
    pub(crate) fn leg_bytes(&self) -> u64 {
        self.legs.as_ref().map_or(0, |legs| legs.owned_bytes())
    }
}
impl Deref for RelationJoinIndex {
    type Target = DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}
impl DerefMut for RelationJoinIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}
impl From<DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>> for RelationJoinIndex {
    fn from(entries: DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>) -> Self {
        Self {
            entries,
            legs: None,
        }
    }
}
impl From<BTreeMap<RelationJoinKey, Vec<RelationJoinEntry>>> for RelationJoinIndex {
    fn from(entries: BTreeMap<RelationJoinKey, Vec<RelationJoinEntry>>) -> Self {
        DerivedIndexEntryMap::from(entries).into()
    }
}
