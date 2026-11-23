//! Join rows and the complete persistent leg state that maintains them.
use super::join_leg_state::JoinLegsState;
use super::{DerivedIndexEntryMap, RelationJoinEntry, RelationJoinKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

type Entries = DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "RelationJoinIndex")]
struct JoinIndexState {
    entries: Entries,
    legs: Option<JoinLegsState>,
}

trait JoinIndexBacking: Send + Sync {
    fn state(&self) -> &JoinIndexState;
    fn state_mut(&mut self) -> &mut JoinIndexState;
}

impl JoinIndexBacking for JoinIndexState {
    fn state(&self) -> &JoinIndexState {
        self
    }

    fn state_mut(&mut self) -> &mut JoinIndexState {
        self
    }
}

/// One retained join generation with its exact member and result paths.
#[derive(Clone)]
pub struct RelationJoinIndex(Arc<dyn JoinIndexBacking>);

impl RelationJoinIndex {
    pub(crate) const fn header_bytes() -> u64 {
        (size_of::<JoinIndexState>() + 2 * size_of::<usize>()) as u64
    }

    pub(crate) fn maintained(entries: Entries, legs: JoinLegsState) -> Self {
        Self(Arc::new(JoinIndexState {
            entries,
            legs: Some(legs),
        }))
    }

    pub(crate) fn legs(&self) -> Option<&JoinLegsState> {
        self.0.state().legs.as_ref()
    }

    pub(crate) fn shares_backing(&self) -> bool {
        Arc::strong_count(&self.0) != 1
    }

    // Owners fund a detached header before editing a shared generation.
    fn edit(&mut self) -> &mut JoinIndexState {
        if self.shares_backing() {
            self.0 = Arc::new(self.0.state().clone());
        }
        Arc::get_mut(&mut self.0)
            .expect("the join owner has exclusive generation custody")
            .state_mut()
    }

    pub(crate) fn edit_entries(&mut self) -> &mut Entries {
        &mut self.edit().entries
    }

    pub(crate) fn install_legs(&mut self, legs: JoinLegsState) {
        self.edit().legs = Some(legs);
    }

    /// cost-lane: cold restore/import. Validate both carried membership axes.
    pub(crate) fn consistent_legs(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) -> bool {
        let Some(legs) = self.legs() else {
            return false;
        };
        legs.left.consistent(lane)
            && legs.right.consistent(lane)
            && self.iter().all(|(key, rows)| {
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
                    self.get(&RelationJoinKey::new(left, right))
                        .is_some_and(|rows| {
                            rows.binary_search_by(|row| row.shared_entity_id().cmp(&shared))
                                .is_ok()
                        })
                })
            })
    }

    pub(crate) fn leg_bytes(&self) -> u64 {
        Self::header_bytes().saturating_add(self.legs().map_or(0, JoinLegsState::owned_bytes))
    }
}

impl Deref for RelationJoinIndex {
    type Target = Entries;

    fn deref(&self) -> &Self::Target {
        &self.0.state().entries
    }
}

#[cfg(test)]
impl std::ops::DerefMut for RelationJoinIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.edit_entries()
    }
}

impl Default for RelationJoinIndex {
    fn default() -> Self {
        Self(Arc::new(JoinIndexState::default()))
    }
}

impl From<Entries> for RelationJoinIndex {
    fn from(entries: Entries) -> Self {
        Self(Arc::new(JoinIndexState {
            entries,
            legs: None,
        }))
    }
}

impl From<BTreeMap<RelationJoinKey, Vec<RelationJoinEntry>>> for RelationJoinIndex {
    fn from(entries: BTreeMap<RelationJoinKey, Vec<RelationJoinEntry>>) -> Self {
        Entries::from(entries).into()
    }
}

impl fmt::Debug for RelationJoinIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.state().fmt(formatter)
    }
}

impl PartialEq for RelationJoinIndex {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.state() == other.0.state()
    }
}
impl Eq for RelationJoinIndex {}

impl Serialize for RelationJoinIndex {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.state().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RelationJoinIndex {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        JoinIndexState::deserialize(deserializer).map(|state| Self(Arc::new(state)))
    }
}
