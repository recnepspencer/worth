use std::collections::BTreeSet;

use crate::identity::data::RelationId;
use crate::storage::partition::AdjacencyDirection;
use crate::transactions::data::EntityReference;
use crate::validation::engine::request::PreparedVisibleRelationEdge;

use super::{relation_scope_details_for_id, RelationIntegrityScopeAccumulator};

impl RelationIntegrityScopeAccumulator<'_, '_, '_, '_> {
    pub(super) fn scan_touched_relations(
        &mut self,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        let mut scanned_sources = BTreeSet::new();
        let mut scanned_targets = BTreeSet::new();
        loop {
            self.check_live()?;
            if let Some((kind, entity_id)) = self.touched_relation_sources.pop_first() {
                if !scanned_sources.insert((kind, entity_id)) {
                    continue;
                }
                let outgoing = self.state_view.try_kind_relation_candidates(
                    entity_id,
                    kind,
                    AdjacencyDirection::Outgoing,
                    self.admit,
                )?;
                self.scan_relation_ids(outgoing)?;
                continue;
            }
            if let Some((kind, entity_id)) = self.touched_relation_targets.pop_first() {
                if !scanned_targets.insert((kind, entity_id)) {
                    continue;
                }
                let incoming = self.state_view.try_kind_relation_candidates(
                    entity_id,
                    kind,
                    AdjacencyDirection::Incoming,
                    self.admit,
                )?;
                self.scan_relation_ids(incoming)?;
                continue;
            }
            break;
        }
        Ok(())
    }

    fn scan_relation_ids(
        &mut self,
        relation_ids: impl IntoIterator<Item = RelationId>,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        for relation_id in relation_ids {
            self.check_live()?;
            if !self.scanned_relations.insert(relation_id)
                || self.deleted_relations.contains(&relation_id)
            {
                continue;
            }
            self.check_live()?;
            let Some((kind_id, source, target)) =
                relation_scope_details_for_id(&self.state_view, relation_id)
            else {
                continue;
            };
            let Some(scope) = self.scopes.get_mut(&kind_id) else {
                continue;
            };
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
            if source_deleted
                || target_deleted
                || self.revalidated_entities.as_ref().is_some_and(|entities| {
                    entities.contains(&source) || entities.contains(&target)
                })
            {
                scope.record_affected_pair(
                    &EntityReference::Existing(source),
                    &EntityReference::Existing(target),
                );
            }
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
                self.touched_relation_sources.insert((kind_id, survivor));
                self.touched_relation_targets.insert((kind_id, survivor));
            }
            self.check_live()?;
        }
        Ok(())
    }
}
