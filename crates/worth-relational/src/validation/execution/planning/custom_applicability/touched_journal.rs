//! Borrowed touch-journal discovery stops before any unfunded metadata read.
use super::{insert_kind, pay_visit, CandidateTouchedKinds};
use crate::identity::data::PartitionId;
use crate::validation::custom_rule::CustomPreparationBudget;
use crate::validation::engine::state_view::InvariantStateView;
use std::ops::ControlFlow;

impl CandidateTouchedKinds {
    pub(super) fn partition(
        &mut self,
        partition: PartitionId,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        pay_visit(budget)?;
        proposed.state().get_partition(partition)?;
        let mut result = Some(());
        let known = proposed
            .state()
            .visit_touched_entity_slots(partition, &mut |slot| {
                result = self.entity_slot(partition, slot, proposed, before, budget);
                flow(result)
            });
        if !known {
            return None;
        }
        result?;
        let known = proposed
            .state()
            .visit_touched_relation_slots(partition, &mut |slot| {
                result = self.relation_slot(partition, slot, proposed, before, budget);
                flow(result)
            });
        known.then_some(())?;
        result
    }

    fn entity_slot(
        &mut self,
        partition: PartitionId,
        slot: usize,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        pay_visit(budget)?;
        let current = proposed.entity_metadata_for_slot(partition, slot);
        let previous = before.and_then(|view| view.entity_metadata_for_slot(partition, slot));
        if current.is_none() && previous.is_none() {
            return None;
        }
        for metadata in [current, previous].into_iter().flatten() {
            insert_kind(&mut self.entities, metadata.kind_id, budget)?;
        }
        Some(())
    }

    fn relation_slot(
        &mut self,
        partition: PartitionId,
        slot: usize,
        proposed: &InvariantStateView<'_>,
        before: Option<&InvariantStateView<'_>>,
        budget: Option<&CustomPreparationBudget>,
    ) -> Option<()> {
        pay_visit(budget)?;
        let current = proposed.relation_metadata_for_slot(partition, slot);
        let previous = before.and_then(|view| view.relation_metadata_for_slot(partition, slot));
        if current.is_none() && previous.is_none() {
            return None;
        }
        for metadata in [current, previous].into_iter().flatten() {
            self.relation_metadata(
                metadata.kind_id,
                metadata.source,
                metadata.target,
                proposed,
                before,
                budget,
            )?;
        }
        Some(())
    }
}

fn flow(result: Option<()>) -> ControlFlow<()> {
    if result.is_some() {
        ControlFlow::Continue(())
    } else {
        ControlFlow::Break(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
    use crate::execution::InvariantPacketAllocations;
    use crate::storage::overlay::{PartitionAccess, WorkingState};
    use crate::validation::custom_rule::CustomPreparationStop;
    use std::collections::BTreeMap;

    #[test]
    fn work_refusal_stops_the_borrowed_journal_before_later_slots() {
        let partition = PartitionId(1);
        let mut working = WorkingState::new(
            BTreeMap::new(),
            AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 4,
            },
        );
        for slot in [9, 3, 7] {
            working.mark_entity_slot_touched(partition, slot);
        }
        let lease = crate::tests::support::test_execution_lease();
        let allocations = InvariantPacketAllocations::new(&lease).unwrap();
        let budget = CustomPreparationBudget::with_scratch_admission(
            lease.status(),
            Some(1),
            allocations.scratch(),
        );
        let mut contacts = 0;
        let mut last_contact = None;
        let mut result = Some(());
        let known = working.visit_touched_entity_slots(partition, &mut |slot| {
            contacts += 1;
            last_contact = Some(slot);
            result = pay_visit(Some(&budget));
            flow(result)
        });
        assert!(known, "stopping does not erase journal presence");
        assert_eq!(result, None, "an unfunded prefix is not a complete capture");
        assert_eq!(contacts, 2, "the later slot must not enter the callback");
        assert_eq!(last_contact, Some(7), "visits remain canonical");
        assert_eq!(budget.stop(), Some(CustomPreparationStop::WorkExhausted));
        let charged = budget.charged_work();
        assert!(pay_visit(Some(&budget)).is_none());
        assert_eq!(budget.charged_work(), charged);
        assert_eq!(budget.stop(), Some(CustomPreparationStop::WorkExhausted));
    }
}
