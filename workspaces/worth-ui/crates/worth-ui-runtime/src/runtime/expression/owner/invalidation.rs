use std::collections::BTreeSet;

use super::evaluation::UiExpressionCompletionReceipt;
use super::operand_binding::UiExpressionInputs;
use super::{UiExpressionRuntimeState, UiExpressionSettlement};
use crate::runtime::expression::UiExpressionSlot;
use crate::runtime::intent::UiIntentApplicationFactUpdateReceipt;

impl UiExpressionRuntimeState {
    /// Re-settles only the readers of the application fact that changed. An
    /// owner that has not followed the active generation settles nothing.
    pub(crate) fn invalidate_application(
        &mut self,
        receipt: &UiIntentApplicationFactUpdateReceipt,
        inputs: &UiExpressionInputs<'_>,
    ) -> UiExpressionSettlement {
        if !self.follows(inputs.generation) {
            return UiExpressionSettlement::default();
        }
        let Some(slot) = inputs.facts.slot_of(receipt.identity()) else {
            return UiExpressionSettlement::default();
        };
        let readers = self.catalog.dependencies().readers_of_application(slot);
        self.counters.index_hits = self
            .counters
            .index_hits
            .saturating_add(readers.len() as u64);
        let dirty = readers.iter().copied().collect();
        self.settle_dirty(dirty, inputs)
    }

    /// Probes only the projection slots some expression reads, and re-settles
    /// the readers of the slots whose retained reference now differs. An
    /// owner that has not followed the active generation settles nothing.
    pub(crate) fn invalidate_published_frame(
        &mut self,
        inputs: &UiExpressionInputs<'_>,
    ) -> UiExpressionSettlement {
        if !self.follows(inputs.generation) {
            return UiExpressionSettlement::default();
        }
        let mut dirty = BTreeSet::new();
        for (slot, retained) in &mut self.projections {
            self.counters.operand_probes = self.counters.operand_probes.saturating_add(1);
            let observed = inputs.mounted.current_projection_input(*slot);
            if observed != *retained {
                *retained = observed;
                let readers = self.catalog.dependencies().readers_of_projection(*slot);
                self.counters.index_hits = self
                    .counters
                    .index_hits
                    .saturating_add(readers.len() as u64);
                dirty.extend(readers.iter().copied());
            }
        }
        self.settle_dirty(dirty, inputs)
    }

    /// Pops the lowest dirty slot until none remain. Slots are topological
    /// ranks, so every upstream expression settles before its readers, and a
    /// reader is pushed only by an upstream outcome that actually changed.
    /// The settlement names exactly the slots counted as published changes.
    pub(super) fn settle_dirty(
        &mut self,
        mut dirty: BTreeSet<UiExpressionSlot>,
        inputs: &UiExpressionInputs<'_>,
    ) -> UiExpressionSettlement {
        let mut changed = BTreeSet::new();
        while let Some(slot) = dirty.pop_first() {
            if let Some(UiExpressionCompletionReceipt::Applied { changed: true }) =
                self.settle(slot, inputs)
            {
                changed.insert(slot);
                let dependents = self.catalog.dependencies().dependents_of(slot);
                self.counters.index_hits = self
                    .counters
                    .index_hits
                    .saturating_add(dependents.len() as u64);
                dirty.extend(dependents.iter().copied());
            }
        }
        UiExpressionSettlement::new(changed)
    }
}
