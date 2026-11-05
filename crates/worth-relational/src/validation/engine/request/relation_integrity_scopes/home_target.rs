//! Complete target judgment includes rows already covered by authored edits.
use super::{RelationIntegrityScopeAccumulator, TransactionCommitError};

impl RelationIntegrityScopeAccumulator<'_, '_, '_, '_> {
    pub(super) fn collect_home_target_source(&mut self) -> Result<(), TransactionCommitError> {
        // Reuse the source incidence accounting and the authored create/delete
        // deltas. These are judgment inputs, not duplicate mutation intents.
        let view = self.state_view.clone();
        for partition in view.state().partition_ids() {
            let control = self.control;
            for slot in view.try_entity_slots(partition, |_, _| control.check())? {
                self.check_live()?;
                if let Some(metadata) = view.entity_metadata_for_slot(partition, slot) {
                    self.collect_revalidated_entity(metadata.entity_id)?;
                }
            }
            for slot in view.try_relation_slots(partition, |_, _| control.check())? {
                self.check_live()?;
                if let Some(metadata) = view.relation_metadata_for_slot(partition, slot) {
                    self.collect_revalidated_relation(metadata.relation_id)?;
                }
            }
        }
        self.check_live()
    }
}
