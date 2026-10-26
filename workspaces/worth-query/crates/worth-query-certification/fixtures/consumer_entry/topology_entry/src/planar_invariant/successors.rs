//! Temporary geometry lookup over the actual complete walk's physical records.
use super::preparation::preparation_checkpoint;
use super::*;
use std::{collections::BTreeMap, mem::size_of};
type Entity<Schema> = WorthQueryApplicationInvariantEntity<Schema, Body>;

pub(super) struct Successors<Schema: TopologyModelBinding> {
    values: BTreeMap<EntityId, Entity<Schema>>,
}
impl<Schema: TopologyModelBinding> Successors<Schema> {
    pub(super) fn from_walk(
        walk: &WorthQueryApplicationInvariantWalk<Schema, PlanarSuccessor, Body>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Self, WorthQueryApplicationInvariantPreparationError> {
        // A BTree node holds at most 11 pairs plus parent/child/header words.
        // Two nodes per actual record conservatively cover insertion splits,
        // including simultaneous old/new nodes. This is a backing grant, not RSS.
        let bytes = size_of::<(EntityId, Entity<Schema>)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(14 * size_of::<usize>()))
            .and_then(|n| n.checked_mul(walk.relations().len()))
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(|| {
                WorthQueryApplicationInvariantPreparationError::new(
                    "planar successor storage overflow",
                )
            })?;
        preparation_checkpoint(budget, 0)?;
        if budget.is_some_and(|budget| !budget.claim_scratch(bytes)) {
            return Err(WorthQueryApplicationInvariantPreparationError::new(
                "planar successor storage refused",
            ));
        }
        let mut values = BTreeMap::new();
        for relation in walk.relations() {
            preparation_checkpoint(budget, lookup_work(values.len()))?;
            if values
                .insert(relation.from().entity_id(), relation.to().clone())
                .is_some()
            {
                return Err(WorthQueryApplicationInvariantPreparationError::new(
                    "multiple planar successor relations",
                ));
            }
        }
        preparation_checkpoint(budget, 0)?;
        Ok(Self { values })
    }
    pub(super) fn next(
        &self,
        entity: &Entity<Schema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Entity<Schema>, WorthQueryApplicationInvariantPreparationError> {
        preparation_checkpoint(budget, lookup_work(self.values.len()))?;
        self.values
            .get(&entity.entity_id())
            .cloned()
            .ok_or_else(|| {
                WorthQueryApplicationInvariantPreparationError::new("planar successor absent")
            })
    }
}
fn lookup_work(count: usize) -> u64 {
    let levels = u64::from(usize::BITS - count.saturating_add(1).leading_zeros()) + 1;
    (11 * levels + 1) * PLANAR_CANDIDATE_WORK
}
