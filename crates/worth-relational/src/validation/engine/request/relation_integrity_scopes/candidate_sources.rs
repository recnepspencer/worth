//! Physical before/after journal projection for relation-law source selection.
use super::successor_scope::reference_quote;
use super::RelationIntegrityScopeAccumulator;
use crate::transactions::data::{EntityReference, TransactionCommitError};
impl RelationIntegrityScopeAccumulator<'_, '_, '_, '_> {
    /// The physical journal includes generated writes as well as authored commands.
    pub(super) fn collect_candidate_slots(&mut self) -> Result<(), TransactionCommitError> {
        for partition in self.partitions.touched_partition_ids().unwrap_or_default() {
            for slot in self
                .partitions
                .touched_entity_slots(partition)
                .unwrap_or_default()
            {
                self.check_live()?;
                let before = self.predecessor.entity_metadata_for_slot(partition, slot);
                let after = self.state_view.entity_metadata_for_slot(partition, slot);
                if before.as_ref().map(|m| (m.entity_id, m.kind_id))
                    == after.as_ref().map(|m| (m.entity_id, m.kind_id))
                {
                    continue;
                }
                for metadata in [before, after].into_iter().flatten() {
                    {
                        for scope in self
                            .scopes
                            .values_mut()
                            .filter(|s| s.requires_visible_successors)
                        {
                            (self.admit)(
                                1,
                                reference_quote(&EntityReference::Existing(metadata.entity_id)),
                            )?;
                            scope
                                .connectivity_sources
                                .insert(EntityReference::Existing(metadata.entity_id));
                        }
                    }
                }
            }
            for slot in self
                .partitions
                .touched_relation_slots(partition)
                .unwrap_or_default()
            {
                self.check_live()?;
                let before = self.predecessor.relation_metadata_for_slot(partition, slot);
                let after = self.state_view.relation_metadata_for_slot(partition, slot);
                if before
                    .as_ref()
                    .map(|m| (m.relation_id, m.kind_id, m.source, m.target))
                    == after
                        .as_ref()
                        .map(|m| (m.relation_id, m.kind_id, m.source, m.target))
                {
                    continue;
                }
                for metadata in [before, after].into_iter().flatten() {
                    if let Some(scope) = self.scopes.get_mut(&metadata.kind_id) {
                        (self.admit)(
                            1,
                            reference_quote(&EntityReference::Existing(metadata.source))
                                .saturating_mul(2),
                        )?;
                        scope.connectivity_sources.extend([
                            EntityReference::Existing(metadata.source),
                            EntityReference::Existing(metadata.target),
                        ]);
                    }
                }
            }
        }
        Ok(())
    }
}
