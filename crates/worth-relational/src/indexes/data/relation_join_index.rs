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
    legs: Option<RetainedJoinLegs>,
}
impl RelationJoinIndex {
    pub(crate) fn maintained(
        entries: DerivedIndexEntryMap<RelationJoinKey, RelationJoinEntry>,
        legs: JoinLegsState,
    ) -> Self {
        Self {
            entries,
            legs: Some(RetainedJoinLegs::new(legs)),
        }
    }
    pub(crate) fn legs(&self) -> Option<&JoinLegsState> {
        self.legs.as_ref().map(RetainedJoinLegs::view)
    }
    pub(crate) fn install_legs(&mut self, legs: JoinLegsState) {
        self.legs = Some(RetainedJoinLegs::new(legs));
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
        self.legs().map_or(0, JoinLegsState::owned_bytes)
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

// This owner checks the exact maintained leg state before a generation can
// share it. Consumers borrow the original maps through the existing Arc.
trait JoinLegBacking: Send + Sync {
    fn view(&self) -> &JoinLegsState;
}

impl JoinLegBacking for JoinLegsState {
    fn view(&self) -> &JoinLegsState {
        self
    }
}

#[derive(Clone)]
struct RetainedJoinLegs(Arc<dyn JoinLegBacking>);

impl RetainedJoinLegs {
    fn new(legs: JoinLegsState) -> Self {
        Self(Arc::new(legs))
    }

    fn view(&self) -> &JoinLegsState {
        self.0.view()
    }
}

impl std::fmt::Debug for RetainedJoinLegs {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.view(), formatter)
    }
}

impl PartialEq for RetainedJoinLegs {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.view() == other.view()
    }
}

impl Eq for RetainedJoinLegs {}

impl Serialize for RetainedJoinLegs {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.view().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RetainedJoinLegs {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        JoinLegsState::deserialize(deserializer).map(Self::new)
    }
}
