//! Exact head-only storage demand for already prepared Native consequences.
use super::WorkingState;
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::storage::overlay::CurrentPartitionAccess;
use std::collections::BTreeSet;
impl WorkingState {
    pub(crate) fn materialize_consequence_records<E>(
        &mut self,
        base: &dyn CurrentPartitionAccess,
        partitions: &BTreeSet<PartitionId>,
        entities: &BTreeSet<EntityId>,
        relations: &BTreeSet<RelationId>,
        mut admit: impl FnMut(u64, Option<u64>) -> Result<(), E>,
    ) -> Result<u64, E> {
        let entity_only = self.clone_mode == super::PartitionCloneMode::EntityOnly;
        if entity_only {
            self.clone_mode = super::PartitionCloneMode::GraphSparseEntities;
        }
        let mut copied = 0u64;
        for &id in partitions {
            if let Some(prior) = base.get_current_partition(id) {
                copied = copied.saturating_add(prior.materialize_consequence_partition(
                    self,
                    entity_only,
                    entities,
                    relations,
                    &mut admit,
                )?);
            }
        }
        Ok(copied)
    }
}
