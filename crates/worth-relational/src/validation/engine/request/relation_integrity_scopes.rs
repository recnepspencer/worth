use std::collections::{BTreeMap, BTreeSet};

use super::RelationScopeRequirement;
use super::{PlannedRelationEdge, PreparedRelationIntegrityScope, PreparedRelationIntegrityScopes};
use crate::config::data::RelationIntegrityScopeBudget;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::overlay::PartitionAccess;
use crate::transactions::data::{EntityReference, MergedCommitPlan};

mod budget;
mod incidence_collection;
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
    accumulator.ensure_budget()?;
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
    minimum_candidate_kinds: BTreeMap<KindId, BTreeSet<KindId>>,
    create_scan_directions: BTreeMap<KindId, (bool, bool, bool, bool)>,
    created_candidate_entities: BTreeSet<crate::transactions::data::CreatedEntityRef>,
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
        let minimum_candidate_kinds = requirements
            .iter()
            .map(|(kind, requirement)| (*kind, requirement.minimum_candidate_kinds.clone()))
            .collect();
        let create_scan_directions = requirements
            .iter()
            .map(|(kind, requirement)| {
                (
                    *kind,
                    (
                        requirement.scan_existing_source_on_create,
                        requirement.scan_existing_target_on_create,
                        requirement.scan_existing_pair_on_create,
                        requirement.scan_full_on_create,
                    ),
                )
            })
            .collect();
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
            minimum_candidate_kinds,
            create_scan_directions,
            created_candidate_entities: BTreeSet::new(),
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
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                self.collect_created_entity(
                    spec.partition_id,
                    spec.kind_id,
                    spec.client_key.clone(),
                )?;
            }
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                self.collect_created_entity(
                    spec.partition_id,
                    spec.kind_id,
                    spec.client_key.clone(),
                )?;
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                for key in &spec.client_keys {
                    self.collect_created_entity(spec.partition_id, spec.kind_id, key.clone())?;
                }
            }
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                self.collect_planned_edge(spec.kind_id, &spec.source, &spec.target)?;
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                self.collect_planned_edge(spec.kind_id, &spec.source, &spec.target)?;
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                for (source, target) in &spec.endpoints {
                    self.collect_planned_edge(spec.kind_id, source, target)?;
                }
            }
            MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::Delete(spec),
            ) => self.collect_relation_delete(spec.relation_id)?,
            MutationIntent::Relation(
                crate::transactions::data::RelationMutationIntent::UpdateEndpoints(spec),
            ) => {
                self.collect_relation_delete(spec.relation_id)?;
                self.collect_planned_edge(spec.kind_id, &spec.source, &spec.target)?;
            }
            MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                self.collect_entity_removal(spec.entity_id)?;
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                self.collect_entity_removal(spec.entity_id)?;
                self.collect_created_entity(
                    spec.replacement.partition_id,
                    spec.replacement.kind_id,
                    spec.replacement.client_key.clone(),
                )?;
            }
            MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
                self.collect_revalidated_entity(spec.entity_id)?;
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

    fn collect_created_entity(
        &mut self,
        partition_id: crate::identity::data::PartitionId,
        kind_id: KindId,
        client_key: crate::symbols::data::ClientKey,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        let created = crate::transactions::data::CreatedEntityRef {
            partition_id,
            kind_id,
            client_key,
        };
        for (relation_kind, kinds) in &self.minimum_candidate_kinds {
            if kinds.contains(&kind_id) {
                self.scopes
                    .get_mut(relation_kind)
                    .expect("minimum scope prepared")
                    .created_candidate_entities
                    .insert(created.clone());
                self.created_candidate_entities.insert(created.clone());
            }
        }
        self.ensure_budget()
    }

    fn collect_revalidated_entity(
        &mut self,
        entity_id: EntityId,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        let Some(metadata) = self.state_view.entity_metadata(entity_id) else {
            return Ok(());
        };
        for (relation_kind, kinds) in &self.minimum_candidate_kinds {
            if kinds.contains(&metadata.kind_id) {
                self.scopes
                    .get_mut(relation_kind)
                    .expect("minimum scope prepared")
                    .minimum_touched_entities
                    .insert(entity_id);
            }
        }
        self.touched_entities.insert(entity_id);
        self.touched_relation_sources.insert(entity_id);
        self.touched_relation_targets.insert(entity_id);
        self.ensure_budget()
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
        let (scan_source, scan_target, scan_pair, scan_full) = self
            .create_scan_directions
            .get(&kind_id)
            .copied()
            .unwrap_or((true, true, true, true));
        let both_existing = matches!(source, EntityReference::Existing(_))
            && matches!(target, EntityReference::Existing(_));
        if scan_source || (scan_pair && both_existing) {
            include_existing_entity_reference(&mut self.touched_relation_sources, source);
        }
        if scan_target {
            include_existing_entity_reference(&mut self.touched_relation_targets, target);
        }
        if scan_full && both_existing {
            include_existing_entity_reference(&mut self.touched_relation_targets, source);
            include_existing_entity_reference(&mut self.touched_relation_sources, target);
        }
        self.ensure_budget()
    }

    fn collect_relation_delete(
        &mut self,
        relation_id: RelationId,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        self.deleted_relations.insert(relation_id);
        if let Some((kind_id, source, target)) =
            relation_scope_details_for_id(&self.state_view, relation_id)
        {
            self.scopes
                .entry(kind_id)
                .or_default()
                .deleted_relation_count += 1;
            self.scopes
                .get_mut(&kind_id)
                .expect("deleted scope prepared")
                .minimum_touched_entities
                .extend([source, target]);
            self.touched_entities.extend([source, target]);
            self.touched_relation_sources.extend([source, target]);
            self.touched_relation_targets.extend([source, target]);
        }
        self.ensure_budget()
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

    fn ensure_budget(&self) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        ensure_relation_integrity_scope_budget(
            self.budget,
            scope_budget_snapshot(
                &self.scopes,
                &self.touched_entities,
                self.created_candidate_entities.len(),
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
