use std::sync::Arc;

use crate::change_source::retained_observation::RelationalRetainedObservation;
use crate::history::data::SharedCanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;

use super::{
    denial::RelationalRecordHistoryDenial as Denial,
    page::{
        RelationalRecordHistoryChange, RelationalRecordHistoryPage,
        RelationalRecordHistoryPageLimits,
    },
    retained_selection::RelationalRecordHistorySelection,
};

#[derive(Debug)]
pub struct RetainedCanonicalRecordHistory {
    pub envelope: SharedCanonicalCommitEnvelope,
    pub retained: RelationalRetainedObservation,
    pub selection: RelationalRecordHistorySelection,
    pub _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub fn prepare_page(
    source: &Arc<RetainedCanonicalRecordHistory>,
    next_record: usize,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    checkpoint: &mut worth_execution::MapKernelContext<'_, '_>,
    limits: RelationalRecordHistoryPageLimits,
) -> Result<RelationalRecordHistoryPage, Denial> {
    checkpoint.poll()?;
    let slot_bytes = std::mem::size_of::<Option<RelationalRecordHistoryChange>>();
    let capacity = limits
        .maximum_entries
        .get()
        .min(limits.maximum_source_visits.get())
        .min(limits.maximum_owned_bytes.get() / slot_bytes);
    if capacity == 0 {
        return Err(Denial::PageStorageTooSmall);
    }
    let mut slots = worth_execution::ExecutionRequestArrayBuilder::allocate(execution, capacity)?;
    let mut next = next_record;
    let mut visits = 0;
    let patches = &source.envelope.patch.authoritative_record_patches;
    while next < patches.len()
        && visits < limits.maximum_source_visits.get()
        && slots.len() < capacity
    {
        charge_visit(checkpoint)?;
        let patch = &patches[next];
        visits += 1;
        if source.includes(patch, checkpoint)? {
            slots.push(Some(RelationalRecordHistoryChange {
                source: Arc::clone(source),
                record_index: next,
            }))?;
        }
        next += 1;
    }
    let entry_count = slots.len();
    while slots.len() < capacity {
        charge_visit(checkpoint)?;
        slots.push(None)?;
    }
    Ok(RelationalRecordHistoryPage {
        slots: slots.seal()?,
        entry_count,
        source_visits: visits,
        next_record: next,
        complete: next == patches.len(),
    })
}

impl RetainedCanonicalRecordHistory {
    fn includes(
        &self,
        patch: &crate::publication::patch::data::PublishedAuthoritativeRecordPatch,
        checkpoint: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<bool, Denial> {
        if self
            .selection
            .subjects()
            .binary_search_by(|subject| subject.record().cmp(&patch.target))
            .is_err()
        {
            return Ok(false);
        }
        // Native may retain an authored update with no surviving aspect delta.
        // Preserve that occurrence without attributing it to any selected key.
        if patch.structural_change != RecordStructuralChange::Updated
            || patch.authoritative_patch.full_grammar_operation_count() == 0
        {
            return Ok(true);
        }
        for aspect in self.selection.aspects() {
            charge_visit(checkpoint)?;
            if patch.authoritative_patch.changes_aspect(aspect) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn charge_visit(checkpoint: &mut worth_execution::MapKernelContext<'_, '_>) -> Result<(), Denial> {
    if checkpoint.has_accounting() {
        checkpoint.checkpoint(1)?;
    } else {
        checkpoint.poll()?;
    }
    Ok(())
}
