mod records;

use std::mem::size_of;

use worth_execution::MapKernelFailure;

use crate::authority::mutation::AdjacencyDelta;
use crate::history::data::RelationalDescriptiveTouch as Touch;
use crate::storage::overlay::{PartitionAccess, PartitionCloneMode};

use super::{Failure, TouchBudget};

/// A native merge publishes a full clone of each touched partition. Compare
/// the complete version columns at every selected slot, including revisions
/// inherited from a parent that had no local patch fragment.
pub(super) fn append(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    adjacency: &[AdjacencyDelta],
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<bool, Failure> {
    if working.clone_mode() != PartitionCloneMode::Full {
        return Ok(false);
    }
    // A branch-reference-only movement has no materialized native edit to
    // prove its graph; Query must verify it from the selected root.
    if working.mutation_journal().is_empty() {
        return Ok(false);
    }
    for (&partition_id, journal) in working.mutation_journal() {
        budget.checkpoint(1)?;
        let previous = selected.get_partition(partition_id);
        let Some(next) = working.get_partition(partition_id) else {
            return Ok(false);
        };
        for &slot in &journal.entity_slots {
            budget.checkpoint(1)?;
            if !records::entity(
                runtime,
                selected,
                working,
                partition_id,
                slot,
                previous.map(|partition| &partition.entity_arena),
                &next.entity_arena,
                touches,
                budget,
            )? {
                return Ok(false);
            }
        }
        for &slot in &journal.relation_slots {
            budget.checkpoint(1)?;
            if !records::relation(
                runtime,
                selected,
                working,
                partition_id,
                slot,
                previous.map(|partition| &partition.relation_arena),
                &next.relation_arena,
                touches,
                budget,
            )? {
                return Ok(false);
            }
        }
        // Ordinary mutation preparation records these changes as canonical
        // adjacency deltas. A journal with unrepresented adjacency edits must
        // not be reported as an exact empty touched graph.
        if (!journal.adjacency_slots.is_empty() || !journal.reverse_adjacency_slots.is_empty())
            && adjacency.is_empty()
            && journal.relation_slots.is_empty()
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn push(
    touches: &mut Vec<Touch>,
    touch: Touch,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<(), Failure> {
    if touches.len() == touches.capacity() {
        let new_capacity = touches
            .capacity()
            .checked_mul(2)
            .unwrap_or(usize::MAX)
            .max(4);
        let growth = new_capacity
            .checked_sub(touches.capacity())
            .and_then(|slots| slots.checked_mul(size_of::<Touch>()))
            .and_then(|bytes| bytes.checked_mul(2))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        budget.claim(growth)?;
        touches
            .try_reserve_exact(new_capacity - touches.len())
            .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    }
    touches.push(touch);
    Ok(())
}
