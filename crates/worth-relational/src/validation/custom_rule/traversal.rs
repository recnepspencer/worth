use std::collections::{BTreeSet, VecDeque};
use std::sync::Mutex;

use crate::identity::data::EntityId;

use super::structural_views::StructuralRelationView;
use crate::validation::data::{CustomInvariantTraversalError, TouchedStructuralSet};

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

        let mut visited_entities = BTreeSet::new();
        let mut traversed_relations = BTreeSet::new();
        let mut queue = VecDeque::new();
        for seed in seeds {
            if visited_entities.insert(*seed) {
                queue.push_back((*seed, 0usize));
            }
        }

        while let Some((entity_id, depth)) = queue.pop_front() {
            if depth >= allowed_depth {
                continue;
            }
            let relation_ids = match direction {
                TraversalDirection::Outgoing => {
                    self.relations.outgoing_relations_for_entity(entity_id)?
                }
                TraversalDirection::Incoming => {
                    self.relations.incoming_relations_for_entity(entity_id)?
                }
            };
            budget.charge_step(relation_ids.len())?;
            if !self.work.try_charge(relation_ids.len().max(1)) {
                return Err(CustomInvariantTraversalError::new(
                    "custom invariant traversal exceeded its work budget",
                ));
            }
            self.performance
                .count_custom_invariant_traversal(0, relation_ids.len());
            for relation_id in relation_ids {
                traversed_relations.insert(relation_id);
                let Some(relation) = self.relations.relation(relation_id) else {
                    continue;
                };
                let next_entity = match direction {
                    TraversalDirection::Outgoing => relation.target,
                    TraversalDirection::Incoming => relation.source,
                };
                if visited_entities.insert(next_entity) {
                    budget.charge_frontier(1)?;
                    self.performance.count_custom_invariant_traversal(1, 0);
                    queue.push_back((next_entity, depth + 1));
                }
            }
        }

        Ok(StructuralTraversalResult {
            visited_entities: visited_entities.into_iter().collect::<Vec<_>>().into(),
            traversed_relations: traversed_relations.into_iter().collect::<Vec<_>>().into(),
            frontier_exhausted: queue.is_empty(),
        })
    }
}

#[derive(Clone, Copy)]
enum TraversalDirection {
    Outgoing,
    Incoming,
}
