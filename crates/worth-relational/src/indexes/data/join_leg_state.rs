//! All leg memberships, including duplicates and currently unpaired endpoints.
use crate::identity::data::{EntityId, RelationId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) type ColdJoinLeg = BTreeMap<EntityId, BTreeMap<EntityId, BTreeSet<RelationId>>>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JoinLegState {
    shared: im::OrdMap<EntityId, im::OrdMap<EntityId, im::OrdSet<RelationId>>>,
    relations: im::OrdMap<RelationId, (EntityId, EntityId)>,
}
impl JoinLegState {
    pub(crate) fn cold(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        rows: ColdJoinLeg,
    ) -> Self {
        let mut state = Self::default();
        for (shared, external) in rows {
            for (external, relations) in external {
                for relation in relations {
                    state.set(relation, Some((shared, external)));
                }
            }
        }
        state
    }
    pub(crate) fn relation_count(&self) -> usize {
        self.relations.len()
    }
    pub(crate) fn coordinate(&self, relation: RelationId) -> Option<(EntityId, EntityId)> {
        self.relations.get(&relation).copied()
    }
    pub(crate) fn minimum(&self, shared: EntityId, external: EntityId) -> Option<RelationId> {
        self.shared.get(&shared)?.get(&external)?.get_min().copied()
    }
    pub(crate) fn members(
        &self,
        shared: EntityId,
    ) -> impl Iterator<Item = (EntityId, RelationId)> + '_ {
        self.shared
            .get(&shared)
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter_map(|(external, rows)| rows.get_min().map(|relation| (*external, *relation)))
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = (RelationId, (EntityId, EntityId))> + '_ {
        self.relations
            .iter()
            .map(|(relation, coordinate)| (*relation, *coordinate))
    }
    pub(crate) fn consistent(
        &self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    ) -> bool {
        let members = self
            .shared
            .values()
            .flat_map(|rows| rows.values())
            .map(im::OrdSet::len)
            .sum::<usize>();
        members == self.relations.len()
            && self.shared.values().all(|rows| {
                !rows.is_empty() && rows.values().all(|relations| !relations.is_empty())
            })
            && self.relations.iter().all(|(relation, (shared, external))| {
                self.shared
                    .get(shared)
                    .and_then(|rows| rows.get(external))
                    .is_some_and(|rows| rows.contains(relation))
            })
    }
    /// Cost: logarithmic paths for one changed relation, never a hub scan.
    pub(crate) fn set(&mut self, relation: RelationId, coordinate: Option<(EntityId, EntityId)>) {
        if self.coordinate(relation) == coordinate {
            return;
        }
        if let Some((shared, external)) = self.relations.remove(&relation)
            && let Some(rows) = self.shared.get_mut(&shared)
        {
            if let Some(relations) = rows.get_mut(&external) {
                relations.remove(&relation);
                if relations.is_empty() {
                    rows.remove(&external);
                }
            }
            if rows.is_empty() {
                self.shared.remove(&shared);
            }
        }
        if let Some((shared, external)) = coordinate {
            self.relations.insert(relation, (shared, external));
            self.shared
                .entry(shared)
                .or_default()
                .entry(external)
                .or_default()
                .insert(relation);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JoinLegsState {
    pub(crate) left: JoinLegState,
    pub(crate) right: JoinLegState,
}
impl JoinLegsState {
    pub(crate) fn cold(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        left: ColdJoinLeg,
        right: ColdJoinLeg,
    ) -> Self {
        Self {
            left: JoinLegState::cold(lane, left),
            right: JoinLegState::cold(lane, right),
        }
    }
    pub(crate) fn relation_count(&self) -> usize {
        self.left
            .relation_count()
            .saturating_add(self.right.relation_count())
    }
    pub(crate) fn owned_bytes(&self) -> u64 {
        // Three scalar persistent membership axes per relation; no field payloads.
        (self.relation_count() as u64)
            .saturating_mul((std::mem::size_of::<(RelationId, EntityId, EntityId)>() * 6) as u64)
    }
}
