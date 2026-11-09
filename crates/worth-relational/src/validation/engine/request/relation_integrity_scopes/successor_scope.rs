//! Change-selected source witnesses over maintained bidirectional kind adjacency.
use super::{planned_incidence::PlannedIncidence, RelationIntegrityScopeAccumulator};
use crate::identity::data::KindId;
use crate::storage::partition::AdjacencyDirection;
use crate::transactions::data::{EntityReference, TransactionCommitError};
use std::collections::BTreeSet;

impl RelationIntegrityScopeAccumulator<'_, '_, '_, '_> {
    pub(super) fn scan_required_visible_successors(
        &mut self,
    ) -> Result<(), TransactionCommitError> {
        let kinds: Vec<_> = self
            .scopes
            .iter()
            .filter_map(|(kind, scope)| {
                (scope.requires_visible_successors && scope.should_execute()).then_some(*kind)
            })
            .collect();
        for kind in kinds {
            let planned = PlannedIncidence::new(&self.scopes[&kind].planned_edges, self.admit)?;
            let mut sources = self.source_seeds(kind)?;
            if self.scopes[&kind].requires_global_evaluation {
                self.expand_predecessors(kind, &mut sources, &planned)?;
            }
            self.capture_successor_closure(kind, sources, &planned)?;
            if self.scopes[&kind].requires_global_evaluation
                && !self.partitions.has_read_misses()
                && !self.predecessor.state().has_read_misses()
            {
                let scope = &self.scopes[&kind];
                crate::inspection::relation_law_sources::record(
                    kind,
                    scope.complete_source_set,
                    &scope.connectivity_sources,
                    &scope.visible_successors,
                );
            }
        }
        Ok(())
    }
    fn source_seeds(
        &self,
        kind: KindId,
    ) -> Result<BTreeSet<EntityReference>, TransactionCommitError> {
        let scope = &self.scopes[&kind];
        let mut sources = BTreeSet::new();
        if scope.requires_global_evaluation {
            for source in &scope.connectivity_sources {
                self.insert_source(&mut sources, source)?;
            }
            for id in scope
                .minimum_touched_entities
                .iter()
                .chain(&self.deleted_entities)
            {
                self.insert_source(&mut sources, &EntityReference::Existing(*id))?;
            }
            for created in &scope.created_candidate_entities {
                (self.admit)(
                    1,
                    reference_layout()
                        .saturating_add(created.client_key.owned_allocation_capacity_bytes()),
                )?;
                sources.insert(EntityReference::Created(created.clone()));
            }
        }
        for edge in &scope.planned_edges {
            self.insert_source(&mut sources, &edge.target)?;
            if scope.requires_global_evaluation {
                self.insert_source(&mut sources, &edge.source)?;
            }
        }
        Ok(sources)
    }
    fn insert_source(
        &self,
        sources: &mut BTreeSet<EntityReference>,
        source: &EntityReference,
    ) -> Result<(), TransactionCommitError> {
        (self.admit)(
            1 + u64::from((sources.len() + 1).ilog2()),
            reference_quote(source),
        )?;
        sources.insert(source.clone());
        Ok(())
    }
    fn reserve_sources(
        &self,
        sources: &BTreeSet<EntityReference>,
        copies: u64,
    ) -> Result<(), TransactionCommitError> {
        // Admit the quote walk before inspecting keys, then the simultaneous
        // owned buffers before any clone/capacity allocation.
        (self.admit)((sources.len() as u64).saturating_mul(copies + 1), 0)?;
        let bytes = sources
            .iter()
            .map(|source| reference_quote(source).saturating_mul(copies))
            .sum();
        (self.admit)(0, bytes)
    }
    fn expand_predecessors(
        &self,
        kind: KindId,
        sources: &mut BTreeSet<EntityReference>,
        planned: &PlannedIncidence,
    ) -> Result<(), TransactionCommitError> {
        // Include the before-image: several removed edges can disconnect an
        // indirect predecessor from every changed endpoint in the postimage.
        self.reserve_sources(sources, 4)?;
        let mut pending: Vec<_> = sources.iter().cloned().collect();
        while let Some(target) = pending.pop() {
            self.check_live()?;
            let mut predecessors = BTreeSet::new();
            if let EntityReference::Existing(id) = target {
                for view in [&self.predecessor, &self.state_view] {
                    for relation in view.try_kind_relation_candidates(
                        id,
                        kind,
                        AdjacencyDirection::Incoming,
                        self.admit,
                    )? {
                        self.check_live()?;
                        self.performance.count_relation_uniqueness_candidates(1);
                        if let Some(edge) = view.relation_metadata(relation)
                            && edge.kind_id == kind
                            && edge.target == id
                        {
                            (self.admit)(
                                1,
                                reference_quote(&EntityReference::Existing(edge.source)),
                            )?;
                            predecessors.insert(EntityReference::Existing(edge.source));
                        }
                    }
                }
            }
            for source in planned.predecessors(&target, self.admit)? {
                (self.admit)(1, reference_quote(source))?;
                predecessors.insert(source.clone());
            }
            for source in predecessors {
                (self.admit)(1, reference_quote(&source).saturating_mul(3))?;
                if sources.insert(source.clone()) {
                    pending.push(source);
                }
            }
        }
        Ok(())
    }
    fn capture_successor_closure(
        &mut self,
        kind: KindId,
        sources: BTreeSet<EntityReference>,
        planned: &PlannedIncidence,
    ) -> Result<(), TransactionCommitError> {
        self.reserve_sources(&sources, 4)?;
        self.scopes.get_mut(&kind).unwrap().connectivity_sources = sources.clone();
        // Every inspected source needs only its forward closure. Unrelated
        // components are never enumerated or materialized into this scope.
        let mut visited = BTreeSet::new();
        let mut pending: Vec<_> = sources.into_iter().collect();
        while let Some(source) = pending.pop() {
            self.check_live()?;
            if !visited.insert(source.clone()) {
                continue;
            }
            let mut targets = BTreeSet::new();
            if let EntityReference::Existing(id) = source {
                for relation in self.state_view.try_kind_relation_candidates(
                    id,
                    kind,
                    AdjacencyDirection::Outgoing,
                    self.admit,
                )? {
                    self.check_live()?;
                    self.performance.count_relation_uniqueness_candidates(1);
                    if self.deleted_relations.contains(&relation) && !self.candidate_is_applied {
                        continue;
                    }
                    if let Some(edge) = self.state_view.relation_metadata(relation)
                        && edge.kind_id == kind
                        && edge.source == id
                        && !self.deleted_entities.contains(&edge.source)
                        && !self.deleted_entities.contains(&edge.target)
                    {
                        (self.admit)(
                            1,
                            reference_quote(&EntityReference::Existing(edge.target))
                                .saturating_mul(6),
                        )?;
                        targets.insert(EntityReference::Existing(edge.target));
                    }
                }
            }
            for target in planned.successors(&source, self.admit)? {
                (self.admit)(1, reference_quote(target).saturating_mul(4))?;
                pending.push(target.clone());
            }
            pending.extend(targets.iter().cloned());
            self.scopes
                .get_mut(&kind)
                .unwrap()
                .visible_successors
                .insert(source, targets.into_iter().collect());
        }
        Ok(())
    }
}

pub(super) fn reference_quote(reference: &EntityReference) -> u64 {
    reference_layout()
        + match reference {
            EntityReference::Existing(_) => 0,
            EntityReference::Created(created) => {
                created.client_key.owned_allocation_capacity_bytes()
            }
        }
}

fn reference_layout() -> u64 {
    (std::mem::size_of::<EntityReference>() + 4 * std::mem::size_of::<usize>()) as u64
}
