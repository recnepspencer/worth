use std::collections::{BTreeMap, BTreeSet};

use super::RelationScopeRequirement;
use super::{
    PlannedRelationEdge, PreparedRelationIntegrityScope, PreparedRelationIntegrityScopes,
    PreparedVisibleRelationEdge,
};
use crate::config::data::RelationIntegrityScopeBudget;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::overlay::PartitionAccess;
use crate::transactions::data::{EntityReference, MergedCommitPlan};

mod budget;
pub(crate) use budget::PreparedRelationIntegrityScopeBudgetExceeded;
use budget::{ensure_relation_integrity_scope_budget, scope_budget_snapshot};

pub(crate) fn prepare_relation_integrity_scopes(
    merged_plan: Option<&MergedCommitPlan>,
    partitions: &dyn PartitionAccess,
    version_id: crate::identity::data::VersionId,
    requirements: BTreeMap<KindId, RelationScopeRequirement>,
    performance: &crate::performance::PerformanceAccess<'_>,
    budget: &RelationIntegrityScopeBudget,
) -> Result<Option<PreparedRelationIntegrityScopes>, PreparedRelationIntegrityScopeBudgetExceeded> {
    if requirements.is_empty() {
        return Ok(None);
    }
    let mut accumulator = RelationIntegrityScopeAccumulator::new(
        partitions,
        version_id,
        requirements,
        performance,
        budget,
    );
    if let Some(merged_plan) = merged_plan {
        accumulator.collect_plan(merged_plan)?;
    }
    accumulator.scan_touched_relations()?;
    accumulator.scan_required_visible_successors()?;
    Ok(accumulator.finish())
}

fn include_existing_entity_reference(
    touched_entities: &mut BTreeSet<EntityId>,
    entity_reference: &EntityReference,
) {
    if let EntityReference::Existing(entity_id) = entity_reference {
        touched_entities.insert(*entity_id);
    }
}

struct RelationIntegrityScopeAccumulator<'access, 'runtime> {
    partitions: &'access dyn PartitionAccess,
    state_view: crate::validation::engine::state_view::InvariantStateView<'access>,
    performance: &'access crate::performance::PerformanceAccess<'runtime>,
    budget: &'access RelationIntegrityScopeBudget,
    scopes: BTreeMap<KindId, PreparedRelationIntegrityScope>,
    touched_entities: BTreeSet<EntityId>,
    touched_relation_sources: BTreeSet<EntityId>,
    touched_relation_targets: BTreeSet<EntityId>,
    deleted_entities: BTreeSet<EntityId>,
    deleted_relations: BTreeSet<RelationId>,
    scanned_relations: BTreeSet<RelationId>,
    planned_edge_count: usize,
}

impl<'access, 'runtime> RelationIntegrityScopeAccumulator<'access, 'runtime> {
    fn new(
        partitions: &'access dyn PartitionAccess,
        version_id: crate::identity::data::VersionId,
        requirements: BTreeMap<KindId, RelationScopeRequirement>,
        performance: &'access crate::performance::PerformanceAccess<'runtime>,
        budget: &'access RelationIntegrityScopeBudget,
    ) -> Self {
        Self {
            partitions,
            state_view: crate::validation::engine::state_view::InvariantStateView::new(
                partitions, version_id,
            ),
            performance,
            budget,
            scopes: requirements
                .into_iter()
                .map(|(kind_id, requirement)| {
                    (
                        kind_id,
                        PreparedRelationIntegrityScope {
                            requires_global_evaluation: requirement.requires_global_evaluation,
                            requires_visible_successors: requirement.requires_visible_successors,
                            ..PreparedRelationIntegrityScope::default()
                        },
                    )
                })
                .collect(),
            touched_entities: BTreeSet::new(),
            touched_relation_sources: BTreeSet::new(),
            touched_relation_targets: BTreeSet::new(),
            deleted_entities: BTreeSet::new(),
            deleted_relations: BTreeSet::new(),
            scanned_relations: BTreeSet::new(),
            planned_edge_count: 0,
        }
    }

    fn collect_plan(
        &mut self,
        plan: &MergedCommitPlan,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        for intent in &plan.merged_intents {
            self.collect_intent(intent)?;
        }
        Ok(())
    }

    fn collect_intent(
        &mut self,
        intent: &crate::transactions::data::MutationIntent,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        use crate::transactions::data::{
            CreateIntent, EntityMutationIntent, MaterializationMutationIntent, MutationIntent,
        };
        match intent {
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                self.collect_planned_edge(spec.kind_id, &spec.source, &spec.target)?;
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                for (source, target) in &spec.endpoints {
                    self.collect_planned_edge(spec.kind_id, source, target)?;
                }
            }
            MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::Delete(spec),
            ) => self.collect_relation_delete(spec.relation_id),
            MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::UpdateEndpoints(spec),
            ) => {
                self.collect_relation_delete(spec.relation_id);
                self.collect_planned_edge(spec.kind_id, &spec.source, &spec.target)?;
            }
            MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                self.collect_entity_removal(spec.entity_id)?;
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                self.collect_entity_removal(spec.entity_id)?;
            }
            MutationIntent::Materialization(
                MaterializationMutationIntent::RematerializeRelation(spec),
            ) => self.collect_planned_edge(
                spec.kind_id,
                &EntityReference::Existing(spec.source),
                &EntityReference::Existing(spec.target),
            )?,
            _ => {}
        }
        Ok(())
    }

    fn collect_planned_edge(
        &mut self,
        kind_id: KindId,
        source: &EntityReference,
        target: &EntityReference,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        self.scopes
            .entry(kind_id)
            .or_default()
            .planned_edges
            .push(PlannedRelationEdge {
                source: source.clone(),
                target: target.clone(),
            });
        self.planned_edge_count += 1;
        include_existing_entity_reference(&mut self.touched_entities, source);
        include_existing_entity_reference(&mut self.touched_entities, target);
        include_existing_entity_reference(&mut self.touched_relation_sources, source);
        include_existing_entity_reference(&mut self.touched_relation_targets, target);
        self.ensure_budget()
    }

    fn collect_relation_delete(&mut self, relation_id: RelationId) {
        self.deleted_relations.insert(relation_id);
        if let Some((kind_id, source, target)) =
            relation_scope_details_for_id(&self.state_view, relation_id)
        {
            self.scopes
                .entry(kind_id)
                .or_default()
                .deleted_relation_count += 1;
            self.touched_entities.extend([source, target]);
            self.touched_relation_sources.insert(source);
            self.touched_relation_targets.insert(target);
        }
    }

    fn collect_entity_removal(
        &mut self,
        entity_id: EntityId,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        self.touched_entities.insert(entity_id);
        self.touched_relation_sources.insert(entity_id);
        self.touched_relation_targets.insert(entity_id);
        self.deleted_entities.insert(entity_id);
        self.ensure_budget()
    }

    fn scan_touched_relations(
        &mut self,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        let entities = self
            .touched_relation_sources
            .union(&self.touched_relation_targets)
            .copied()
            .collect::<Vec<_>>();
        for entity_id in entities {
            let Some(partition) = self.partitions.get_partition(entity_id.partition_id) else {
                continue;
            };
            let slot = entity_id.slot_index();
            let outgoing = partition
                .adjacency
                .get(slot)
                .map(|set| set.current_ids().to_vec())
                .unwrap_or_default();
            self.scan_relation_ids(outgoing)?;
            let incoming = partition
                .reverse_adjacency
                .get(slot)
                .map(|set| set.current_ids().to_vec())
                .unwrap_or_default();
            self.scan_relation_ids(incoming)?;
        }
        Ok(())
    }

    fn scan_relation_ids(
        &mut self,
        relation_ids: impl IntoIterator<Item = RelationId>,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        for relation_id in relation_ids {
            if !self.scanned_relations.insert(relation_id)
                || self.deleted_relations.contains(&relation_id)
            {
                continue;
            }
            self.ensure_budget()?;
            let Some((kind_id, source, target)) =
                relation_scope_details_for_id(&self.state_view, relation_id)
            else {
                continue;
            };
            let scope = self.scopes.entry(kind_id).or_default();
            scope.visible_edges.push(PreparedVisibleRelationEdge {
                relation_id,
                source,
                target,
            });
            scope.increment_counts(
                EntityReference::Existing(source),
                EntityReference::Existing(target),
            );
            self.performance.count_relation_uniqueness_candidates(1);
            if self.deleted_entities.contains(&source) {
                scope.deleted_entities.insert(source);
            }
            if self.deleted_entities.contains(&target) {
                scope.deleted_entities.insert(target);
            }
        }
        Ok(())
    }

    fn scan_required_visible_successors(
        &mut self,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        let required_kinds = self
            .scopes
            .iter()
            .filter_map(|(kind_id, scope)| {
                (scope.requires_visible_successors && scope.should_execute()).then_some(*kind_id)
            })
            .collect::<BTreeSet<_>>();
        if required_kinds.is_empty() {
            return Ok(());
        }
        for partition_id in self.state_view.state().partition_ids() {
            let slot_count = self
                .state_view
                .relation_slot_scan_count(partition_id)
                .unwrap_or_default();
            for slot in 0..slot_count {
                let Some(metadata) = self
                    .state_view
                    .relation_metadata_for_slot(partition_id, slot)
                else {
                    continue;
                };
                if !required_kinds.contains(&metadata.kind_id)
                    || self.deleted_relations.contains(&metadata.relation_id)
                    || self.deleted_entities.contains(&metadata.source)
                    || self.deleted_entities.contains(&metadata.target)
                {
                    continue;
                }
                self.scanned_relations.insert(metadata.relation_id);
                self.ensure_budget()?;
                self.scopes
                    .get_mut(&metadata.kind_id)
                    .expect("required relation kind scope must be prepared")
                    .record_visible_successor(
                        EntityReference::Existing(metadata.source),
                        EntityReference::Existing(metadata.target),
                    );
            }
        }
        for scope in self.scopes.values_mut() {
            for successors in scope.visible_successors.values_mut() {
                successors.sort();
                successors.dedup();
            }
        }
        Ok(())
    }

    fn ensure_budget(&self) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        ensure_relation_integrity_scope_budget(
            self.budget,
            scope_budget_snapshot(
                &self.scopes,
                &self.touched_entities,
                &self.deleted_entities,
                &self.scanned_relations,
                self.planned_edge_count,
            ),
        )
    }

    fn finish(mut self) -> Option<PreparedRelationIntegrityScopes> {
        for scope in self.scopes.values_mut() {
            let planned_edges = std::mem::take(&mut scope.planned_edges);
            for edge in planned_edges {
                scope.increment_counts(edge.source.clone(), edge.target.clone());
                for entity in [&edge.source, &edge.target] {
                    if let EntityReference::Existing(entity_id) = entity {
                        if self.deleted_entities.contains(entity_id) {
                            scope.deleted_entities.insert(*entity_id);
                        }
                    }
                }
                scope.planned_edges.push(edge);
            }
        }
        self.scopes.retain(|_, scope| scope.should_execute());
        (!self.scopes.is_empty()).then(|| PreparedRelationIntegrityScopes::new(self.scopes))
    }
}

fn relation_scope_details_for_id(
    state_view: &crate::validation::engine::state_view::InvariantStateView<'_>,
    relation_id: RelationId,
) -> Option<(KindId, EntityId, EntityId)> {
    let metadata = state_view.relation_metadata(relation_id)?;
    Some((metadata.kind_id, metadata.source, metadata.target))
}
