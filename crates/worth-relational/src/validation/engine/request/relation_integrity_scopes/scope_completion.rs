use super::{
    PreparedRelationIntegrityScopes, RelationIntegrityScopeAccumulator, TransactionCommitError,
};
use crate::transactions::data::EntityReference;

impl RelationIntegrityScopeAccumulator<'_, '_, '_, '_> {
    pub(super) fn finish(
        mut self,
    ) -> Result<Option<PreparedRelationIntegrityScopes>, TransactionCommitError> {
        self.check_live()?;
        for scope in self.scopes.values_mut() {
            self.control.check()?;
            for entity in scope
                .minimum_touched_entities
                .intersection(&self.deleted_entities)
            {
                self.control.check()?;
                scope.deleted_entities.insert(*entity);
            }
            let planned_edges = std::mem::take(&mut scope.planned_edges);
            for edge in planned_edges {
                self.control.check()?;
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
        self.check_live()?;
        Ok((!self.scopes.is_empty()).then(|| PreparedRelationIntegrityScopes::new(self.scopes)))
    }
}
