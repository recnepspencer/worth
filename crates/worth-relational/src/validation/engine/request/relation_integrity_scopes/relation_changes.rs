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
        let Some(scope) = self.scopes.get_mut(&kind_id) else {
            return Ok(());
        };
        scope.record_affected_pair(source, target);
        if scope.requires_visible_successors {
            (self.admit)(
                1,
                super::successor_scope::reference_quote(source)
                    .saturating_add(super::successor_scope::reference_quote(target)),
            )?;
            scope
                .connectivity_sources
                .extend([source.clone(), target.clone()]);
        }
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
            include_existing_entity_reference(&mut self.touched_relation_sources, kind_id, source);
        }
        if scan_target {
            include_existing_entity_reference(&mut self.touched_relation_targets, kind_id, target);
        }
        if scan_full && both_existing {
            include_existing_entity_reference(&mut self.touched_relation_targets, kind_id, source);
            include_existing_entity_reference(&mut self.touched_relation_sources, kind_id, target);
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
            let Some(scope) = self.scopes.get_mut(&kind) else {
                return Ok(());
            };
            scope.record_affected_pair(
                &EntityReference::Existing(source),
                &EntityReference::Existing(target),
            );
            scope.minimum_touched_entities.extend([source, target]);
            self.touched_relation_sources
                .extend([(kind, source), (kind, target)]);
            self.touched_relation_targets
                .extend([(kind, source), (kind, target)]);
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
            relation_scope_details_for_id(&self.predecessor, relation_id)
                .or_else(|| relation_scope_details_for_id(&self.state_view, relation_id))
        {
            let Some(scope) = self.scopes.get_mut(&kind_id) else {
                return Ok(());
            };
            scope.record_affected_pair(
                &EntityReference::Existing(source),
                &EntityReference::Existing(target),
            );
            scope.deleted_relation_count += 1;
            scope.minimum_touched_entities.extend([source, target]);
            self.touched_relation_sources
                .extend([(kind_id, source), (kind_id, target)]);
            self.touched_relation_targets
                .extend([(kind_id, source), (kind_id, target)]);
        }
        self.check_live()
    }
}
