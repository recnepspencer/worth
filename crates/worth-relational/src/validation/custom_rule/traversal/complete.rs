//! Complete one-kind reads on the native candidate/before-image view.
use crate::identity::data::{EntityId, KindId};

use super::super::StructuralReadError;
use crate::structural_traversal::{self as breadth_first, BreadthFirstRead};
mod retention_extent;
use super::complete_result::*;
use super::{StructuralRelationView, TraversalDirection};

impl StructuralRelationView<'_> {
    /// Complete outgoing reachability, including roots at distance zero.
    ///
    /// O((v+a) log(v+a)) time and O(v+e) retained space, where a includes
    /// adjacency candidates at visited vertices and e is matching edges.
    /// Reads use this view's original meter and the caller's original request.
    pub fn walk_outgoing_from(
        &self,
        seeds: &[EntityId],
        relation_kind: KindId,
    ) -> Result<CompleteStructuralWalk, StructuralWalkDenial> {
        self.walk_complete(seeds, relation_kind, TraversalDirection::Outgoing)
    }

    /// Complete incoming reachability with the same ordering, authority and
    /// cost contract as `walk_outgoing_from`.
    pub fn walk_incoming_from(
        &self,
        seeds: &[EntityId],
        relation_kind: KindId,
    ) -> Result<CompleteStructuralWalk, StructuralWalkDenial> {
        self.walk_complete(seeds, relation_kind, TraversalDirection::Incoming)
    }

    fn walk_complete(
        &self,
        seeds: &[EntityId],
        relation_kind: KindId,
        direction: TraversalDirection,
    ) -> Result<CompleteStructuralWalk, StructuralWalkDenial> {
        let mut read = CompleteRead {
            view: self,
            relation_kind,
            direction,
            result: CompleteStructuralWalk {
                steps: Vec::new(),
                relations: Vec::new(),
                reads: StructuralWalkReads::default(),
                _retention: self.work_meter(),
            },
        };
        read.check_live()?;
        self.require_relation_kind(relation_kind)
            .map_err(|error| read.refusal(error))?;
        breadth_first::walk(seeds, &mut read)?;
        read.check_live()?;
        read.result
            .steps
            .sort_unstable_by_key(|step| (step.distance, step.entity));
        read.result
            .relations
            .sort_unstable_by_key(|record| record.relation_id);
        read.check_live()?;
        Ok(read.result)
    }
}

struct CompleteRead<'read, 'runtime> {
    view: &'read StructuralRelationView<'runtime>,
    relation_kind: KindId,
    direction: TraversalDirection,
    result: CompleteStructuralWalk,
}

impl CompleteRead<'_, '_> {
    fn check_live(&self) -> Result<(), StructuralWalkDenial> {
        self.view
            .read_budget()
            .check_live()
            .map_err(|kind| self.denial(kind))
    }

    fn refusal(&self, error: StructuralReadError) -> StructuralWalkDenial {
        self.denial(self.view.read_budget().refusal(error))
    }

    fn denial(&self, kind: StructuralWalkDenialKind) -> StructuralWalkDenial {
        StructuralWalkDenial {
            kind,
            reads: self.result.reads,
        }
    }

    fn claim(&self, bytes: usize) -> Result<(), StructuralWalkDenial> {
        self.view
            .read_budget()
            .claim_retained(bytes as u64)
            .map_err(|kind| self.denial(kind))
    }
}

impl BreadthFirstRead for CompleteRead<'_, '_> {
    type Error = StructuralWalkDenial;

    fn seed(&mut self, entity: EntityId) -> Result<(), Self::Error> {
        self.view
            .read_budget()
            .checkpoint(1)
            .map_err(|kind| self.denial(kind))?;
        #[cfg(test)]
        self.view.record_walk_seed_for_test(entity);
        #[cfg(not(test))]
        let _ = entity;
        self.check_live()
    }

    fn retain(&mut self, _entity: EntityId, _distance: usize) -> Result<(), Self::Error> {
        self.check_live()?;
        // BTree visited/frontier entries and result Vec growth coexist. Use
        // the original structural payer before growing any of those owners.
        self.claim(retention_extent::vertex_bytes())?;
        self.view
            .read_budget()
            .checkpoint(1)
            .map_err(|kind| self.denial(kind))
    }

    fn expand(
        &mut self,
        entity: EntityId,
        distance: usize,
        mut enqueue: impl FnMut(&mut Self, EntityId) -> Result<(), Self::Error>,
    ) -> Result<(), Self::Error> {
        self.check_live()?;
        let kind = self
            .view
            .entity_kind_observed(entity, || {
                self.result.reads.entity_records += 1;
            })
            .map_err(|error| self.refusal(error))?;
        self.result.steps.push(StructuralWalkStep {
            entity,
            kind,
            distance,
        });
        #[cfg(test)]
        self.view.record_walk_visit_for_test(entity);
        self.check_live()?;
        let ids = match self.direction {
            TraversalDirection::Outgoing => self.view.outgoing_relations_for_entity(entity),
            TraversalDirection::Incoming => self.view.incoming_relations_for_entity(entity),
        }
        .map_err(|_| self.refusal(StructuralReadError::WorkBudgetExceeded))?;
        self.result.reads.adjacency_lists += 1;
        for id in ids {
            self.check_live()?;
            let record = self
                .view
                .relation(id)
                .map_err(|error| self.refusal(error))?;
            self.result.reads.relation_records += 1;
            if record.kind_id != self.relation_kind {
                continue;
            }
            self.claim(retention_extent::relation_bytes())?;
            let neighbor = match self.direction {
                TraversalDirection::Outgoing => record.target,
                TraversalDirection::Incoming => record.source,
            };
            self.result.relations.push(record);
            enqueue(self, neighbor)?;
        }
        Ok(())
    }
}
