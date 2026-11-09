//! Preserve already-applied writers while admitting the later writer's real paths.
use super::WorkingState;
use crate::storage::overlay::PartitionCloneMode;
impl WorkingState {
    pub(crate) fn adopt_sequential_apply(
        &mut self,
        next: Self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        self.clone_mode = PartitionCloneMode::GraphSparseEntities;
        self.content = next.content;
        for (id, mut overlay) in next.partitions {
            request.consult()?;
            let journal = next.mutation_journal.get(&id).cloned().unwrap_or_default();
            let adjacency_omitted = if self.partitions.contains_key(&id) {
                self.mutation_journal
                    .get(&id)
                    .is_some_and(|j| j.adjacency_omitted)
            } else {
                journal.adjacency_omitted
            };
            if let Some(prior) = self.partitions.get_mut_for_request(&id, request)? {
                prior.merge_overlay_from_owned(&mut overlay, &journal);
                prior.relation_overlay_is_sparse = true;
            } else {
                overlay.relation_overlay_is_sparse = true;
                self.partitions.insert_for_request(id, overlay, request)?;
            }
            if !self.mutation_journal.contains_key(&id) {
                self.mutation_journal
                    .insert_for_request(id, Default::default(), request)?;
            }
            let combined = self
                .mutation_journal
                .get_mut_for_request(&id, request)?
                .expect("the same exclusively borrowed working state installed this journal");
            combined.adjacency_omitted = adjacency_omitted;
            for (target, source) in [
                (&mut combined.entity_slots, &journal.entity_slots),
                (&mut combined.relation_slots, &journal.relation_slots),
                (&mut combined.adjacency_slots, &journal.adjacency_slots),
                (
                    &mut combined.reverse_adjacency_slots,
                    &journal.reverse_adjacency_slots,
                ),
            ] {
                for &slot in source {
                    target.insert_for_request(slot, request)?;
                }
            }
        }
        Ok(())
    }
}
