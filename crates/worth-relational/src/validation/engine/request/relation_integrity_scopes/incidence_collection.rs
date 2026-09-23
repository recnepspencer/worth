use std::collections::BTreeSet;

use crate::identity::data::RelationId;
use crate::transactions::data::EntityReference;
use crate::validation::engine::request::PreparedVisibleRelationEdge;

use super::{
    relation_scope_details_for_id, PreparedRelationIntegrityScopeBudgetExceeded,
    RelationIntegrityScopeAccumulator,
};

impl RelationIntegrityScopeAccumulator<'_, '_> {
    pub(super) fn scan_touched_relations(
        &mut self,
    ) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
        let mut scanned_sources = BTreeSet::new();
        let mut scanned_targets = BTreeSet::new();
        loop {
            if let Some(entity_id) = self.touched_relation_sources.pop_first() {
                if !scanned_sources.insert(entity_id) {
                    continue;
                }
                if let Some(partition) = self.partitions.get_partition(entity_id.partition_id) {
                    let outgoing = partition
                        .adjacency
                        .get(entity_id.slot_index())
                        .map(|set| set.current_ids().to_vec())
                        .unwrap_or_default();
                    self.scan_relation_ids(outgoing)?;
                }
                continue;
            }
            if let Some(entity_id) = self.touched_relation_targets.pop_first() {
                if !scanned_targets.insert(entity_id) {
                    continue;
                }
                if let Some(partition) = self.partitions.get_partition(entity_id.partition_id) {
                    let incoming = partition
                        .reverse_adjacency
                        .get(entity_id.slot_index())
                        .map(|set| set.current_ids().to_vec())
                        .unwrap_or_default();
                    self.scan_relation_ids(incoming)?;
                }
                continue;
            }
            break;
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
            let source_deleted = self.deleted_entities.contains(&source);
            let target_deleted = self.deleted_entities.contains(&target);
            if source_deleted {
                scope.deleted_entities.insert(source);
            }
            if target_deleted {
                scope.deleted_entities.insert(target);
            }
            let survivor = match (source_deleted, target_deleted) {
                (true, false) => Some(target),
                (false, true) => Some(source),
                _ => None,
            };
            if let Some(survivor) = survivor.filter(|_| {
                self.minimum_candidate_kinds
                    .get(&kind_id)
                    .is_some_and(|kinds| !kinds.is_empty())
            }) {
                scope.minimum_touched_entities.insert(survivor);
                self.touched_entities.insert(survivor);
                self.touched_relation_sources.insert(survivor);
                self.touched_relation_targets.insert(survivor);
            }
            self.ensure_budget()?;
        }
        Ok(())
    }

    pub(super) fn scan_required_visible_successors(
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
}
