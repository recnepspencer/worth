use std::collections::BTreeSet;
use std::sync::Mutex;

use crate::identity::data::EntityId;

use super::structural_views::StructuralRelationView;
use crate::validation::data::{CustomInvariantTraversalError, TouchedStructuralSet};

mod breadth_first;
mod complete;
mod complete_result;
pub use complete_result::{
    CompleteStructuralWalk, StructuralWalkDenial, StructuralWalkDenialKind, StructuralWalkReads,
    StructuralWalkStep,
};
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TraversalBudgetSession {
    remaining_frontier: usize,
    remaining_steps: usize,
    max_depth: usize,
    consumed_frontier: usize,
    consumed_steps: usize,
}

impl TraversalBudgetSession {
    fn from_touched_scope(touched: &TouchedStructuralSet) -> Self {
        let base_entities =
            touched.visible_entity_ids().len() + touched.planned_entity_creates().len();
        let base_relations =
            touched.visible_relation_ids().len() + touched.planned_relation_creates().len();
        let base = (base_entities + base_relations).max(32);
        Self {
            remaining_frontier: base.saturating_mul(8),
            remaining_steps: base.saturating_mul(32),
            max_depth: 32,
            consumed_frontier: 0,
            consumed_steps: 0,
        }
    }

    fn charge_frontier(&mut self, units: usize) -> Result<(), CustomInvariantTraversalError> {
        if units > self.remaining_frontier {
            return Err(CustomInvariantTraversalError::new(
                "custom invariant traversal exceeded its session frontier budget",
            ));
        }
        self.remaining_frontier -= units;
        self.consumed_frontier += units;
        Ok(())
    }

    fn charge_step(&mut self, units: usize) -> Result<(), CustomInvariantTraversalError> {
        if units > self.remaining_steps {
            return Err(CustomInvariantTraversalError::new(
                "custom invariant traversal exceeded its session step budget",
            ));
        }
        self.remaining_steps -= units;
        self.consumed_steps += units;
        Ok(())
    }

    fn checked_depth(
        &self,
        requested_depth: usize,
    ) -> Result<usize, CustomInvariantTraversalError> {
        if requested_depth > self.max_depth {
            return Err(CustomInvariantTraversalError::new(format!(
                "custom invariant traversal requested depth {} beyond session maximum {}",
                requested_depth, self.max_depth
            )));
        }
        Ok(requested_depth)
    }

    fn summary(&self) -> CustomInvariantTraversalSummary {
        CustomInvariantTraversalSummary {
            consumed_frontier: self.consumed_frontier,
            consumed_steps: self.consumed_steps,
            remaining_frontier: self.remaining_frontier,
            remaining_steps: self.remaining_steps,
            max_depth: self.max_depth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomInvariantTraversalSummary {
    pub consumed_frontier: usize,
    pub consumed_steps: usize,
    pub remaining_frontier: usize,
    pub remaining_steps: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralTraversalResult {
    visited_entities: Arc<[EntityId]>,
    traversed_relations: Arc<[crate::identity::data::RelationId]>,
    frontier_exhausted: bool,
}

use std::sync::Arc;

impl StructuralTraversalResult {
    pub fn visited_entities(&self) -> &[EntityId] {
        &self.visited_entities
    }

    pub fn traversed_relations(&self) -> &[crate::identity::data::RelationId] {
        &self.traversed_relations
    }

    pub fn frontier_exhausted(&self) -> bool {
        self.frontier_exhausted
    }
}

pub struct BoundedStructuralTraversal<'runtime> {
    relations: StructuralRelationView<'runtime>,
    performance: crate::performance::PerformanceAccess<'runtime>,
    budget: Mutex<TraversalBudgetSession>,
    work: super::CustomInvariantWorkMeter,
}

impl<'runtime> BoundedStructuralTraversal<'runtime> {
    pub(crate) fn new(
        performance: crate::performance::PerformanceAccess<'runtime>,
        relations: StructuralRelationView<'runtime>,
        touched: &TouchedStructuralSet,
        work: super::CustomInvariantWorkMeter,
    ) -> Self {
        Self {
            relations,
            performance,
            budget: Mutex::new(TraversalBudgetSession::from_touched_scope(touched)),
            work,
        }
    }

    pub fn walk_outgoing_from(
        &self,
        seeds: &[EntityId],
        max_depth: usize,
    ) -> Result<StructuralTraversalResult, CustomInvariantTraversalError> {
        self.walk(seeds, max_depth, TraversalDirection::Outgoing)
    }

    pub fn walk_incoming_from(
        &self,
        seeds: &[EntityId],
        max_depth: usize,
    ) -> Result<StructuralTraversalResult, CustomInvariantTraversalError> {
        self.walk(seeds, max_depth, TraversalDirection::Incoming)
    }

    pub(crate) fn summary(&self) -> CustomInvariantTraversalSummary {
        self.budget
            .lock()
            .expect("custom invariant traversal budget mutex must not be poisoned")
            .summary()
    }

    fn walk(
        &self,
        seeds: &[EntityId],
        max_depth: usize,
        direction: TraversalDirection,
    ) -> Result<StructuralTraversalResult, CustomInvariantTraversalError> {
        let mut budget = self
            .budget
            .lock()
            .expect("custom invariant traversal budget mutex must not be poisoned");
        let allowed_depth = budget.checked_depth(max_depth)?;
        budget.charge_frontier(seeds.len())?;
        if !self.work.try_charge(seeds.len().max(1)) {
            return Err(CustomInvariantTraversalError::new(
                "custom invariant frontier exceeded its work budget",
            ));
        }
        self.performance
            .count_custom_invariant_traversal(seeds.len(), 0);

        let mut read = ScopeRead {
            owner: self,
            budget: &mut budget,
            allowed_depth,
            direction,
            traversed_relations: BTreeSet::new(),
        };
        let visited_entities = breadth_first::walk(seeds, &mut read)?;
        let traversed_relations = read.traversed_relations;

        Ok(StructuralTraversalResult {
            visited_entities: visited_entities.into_iter().collect::<Vec<_>>().into(),
            traversed_relations: traversed_relations.into_iter().collect::<Vec<_>>().into(),
            frontier_exhausted: true,
        })
    }
}

#[derive(Clone, Copy)]
enum TraversalDirection {
    Outgoing,
    Incoming,
}

struct ScopeRead<'read, 'runtime> {
    owner: &'read BoundedStructuralTraversal<'runtime>,
    budget: &'read mut TraversalBudgetSession,
    allowed_depth: usize,
    direction: TraversalDirection,
    traversed_relations: BTreeSet<crate::identity::data::RelationId>,
}

impl breadth_first::BreadthFirstRead for ScopeRead<'_, '_> {
    type Error = CustomInvariantTraversalError;

    fn retain(&mut self, _entity: EntityId, distance: usize) -> Result<(), Self::Error> {
        // Legacy scope policy charges the raw seed list once, including duplicates.
        if distance != 0 {
            self.budget.charge_frontier(1)?;
            self.owner
                .performance
                .count_custom_invariant_traversal(1, 0);
        }
        Ok(())
    }

    fn expand(
        &mut self,
        entity: EntityId,
        distance: usize,
        mut enqueue: impl FnMut(&mut Self, EntityId) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error> {
        if distance >= self.allowed_depth {
            return Ok(());
        }
        let ids = match self.direction {
            TraversalDirection::Outgoing => {
                self.owner.relations.outgoing_relations_for_entity(entity)?
            }
            TraversalDirection::Incoming => {
                self.owner.relations.incoming_relations_for_entity(entity)?
            }
        };
        self.budget.charge_step(ids.len())?;
        if !self.owner.work.try_charge(ids.len().max(1)) {
            return Err(CustomInvariantTraversalError::new(
                "custom invariant traversal exceeded its work budget",
            ));
        }
        self.owner
            .performance
            .count_custom_invariant_traversal(0, ids.len());
        for id in ids {
            self.traversed_relations.insert(id);
            let relation = self.owner.relations.relation(id).map_err(|error| {
                CustomInvariantTraversalError::new(format!(
                    "custom invariant relation read failed: {error:?}"
                ))
            })?;
            let neighbor = match self.direction {
                TraversalDirection::Outgoing => relation.target,
                TraversalDirection::Incoming => relation.source,
            };
            enqueue(self, neighbor)?;
        }
        Ok(())
    }
}
