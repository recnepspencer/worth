//! Changed and explicitly revalidated relation pairs define native judgment scope.
use super::{
    include_existing_entity_reference, relation_scope_details_for_id, PlannedRelationEdge,
    RelationIntegrityScopeAccumulator, TransactionCommitError,
};
use crate::identity::data::{KindId, RelationId};
use crate::transactions::data::EntityReference;

impl RelationIntegrityScopeAccumulator<'_, '_, '_, '_> {
    pub(super) fn collect_planned_edge(
        &mut self,
        kind_id: KindId,
        source: &EntityReference,
        target: &EntityReference,
    ) -> Result<(), TransactionCommitError> {
        self.check_live()?;
        let scope = self.scopes.entry(kind_id).or_default();
        scope.record_affected_pair(source, target);
        scope.planned_edges.push(PlannedRelationEdge {
            source: source.clone(),
            target: target.clone(),
        });
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
        self.check_live()
    }

    pub(super) fn collect_revalidated_relation(
        &mut self,
        relation_id: RelationId,
    ) -> Result<(), TransactionCommitError> {
        self.check_live()?;
        if let Some((kind, source, target)) =
            relation_scope_details_for_id(&self.state_view, relation_id)
        {
            let scope = self.scopes.entry(kind).or_default();
            scope.record_affected_pair(
                &EntityReference::Existing(source),
                &EntityReference::Existing(target),
            );
            scope.minimum_touched_entities.extend([source, target]);
            self.touched_relation_sources.extend([source, target]);
            self.touched_relation_targets.extend([source, target]);
        }
        self.check_live()
    }

    pub(super) fn collect_relation_delete(
        &mut self,
        relation_id: RelationId,
    ) -> Result<(), TransactionCommitError> {
        self.check_live()?;
        self.deleted_relations.insert(relation_id);
        if let Some((kind_id, source, target)) =
            relation_scope_details_for_id(&self.state_view, relation_id)
        {
            let scope = self.scopes.entry(kind_id).or_default();
            scope.record_affected_pair(
                &EntityReference::Existing(source),
                &EntityReference::Existing(target),
            );
            scope.deleted_relation_count += 1;
            scope.minimum_touched_entities.extend([source, target]);
            self.touched_relation_sources.extend([source, target]);
            self.touched_relation_targets.extend([source, target]);
        }
        self.check_live()
    }
}
