//! Selected graph edges enter the same relation-integrity scope accumulator.
use super::*;
impl RelationIntegrityScopeAccumulator<'_, '_, '_, '_> {
    pub(super) fn collect_owned_graph(
        &mut self,
        selection: &crate::history::data::CanonicalOwnedGraphSelection,
    ) -> Result<(), TransactionCommitError> {
        for id in selection.retired_entities() {
            self.collect_entity_removal(id)?;
        }
        for (id, kind) in selection.selected_entities() {
            self.check_live()?;
            if let Some(entities) = &mut self.revalidated_entities {
                entities.insert(id);
            }
            for (relation_kind, kinds) in &self.minimum_candidate_kinds {
                self.check_live()?;
                if kinds.contains(&kind) {
                    self.scopes
                        .get_mut(relation_kind)
                        .expect("prepared requirement scope")
                        .minimum_touched_entities
                        .insert(id);
                }
            }
            self.touched_relation_sources.insert(id);
            self.touched_relation_targets.insert(id);
        }
        for edge in selection.current_relations() {
            self.collect_relation_delete(edge.relation_id)?;
        }
        for edge in selection.selected_relations() {
            self.collect_planned_edge(
                edge.kind_id,
                &EntityReference::Existing(edge.source),
                &EntityReference::Existing(edge.target),
            )?;
        }
        Ok(())
    }
}
