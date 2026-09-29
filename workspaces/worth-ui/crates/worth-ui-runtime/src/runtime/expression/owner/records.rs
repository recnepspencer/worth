use super::UiExpressionEvaluationRecord;
use crate::runtime::expression::{UiExpressionSlot, UiExpressionSlotCount};

/// The retained record of every installed expression, one cell per slot. A
/// record is stored in the cell of its own slot, so it cannot land in
/// another expression's position; a cell stays empty until its expression is
/// admitted.
pub(super) struct UiExpressionRecords(Box<[Option<UiExpressionEvaluationRecord>]>);

impl UiExpressionRecords {
    pub(super) fn unsettled(slots: UiExpressionSlotCount) -> Self {
        Self(slots.slots().map(|_| None).collect())
    }

    pub(super) fn get(&self, slot: UiExpressionSlot) -> Option<&UiExpressionEvaluationRecord> {
        self.0.get(slot.index())?.as_ref()
    }

    /// Removes and returns the record of `slot`, leaving its cell empty.
    pub(super) fn take(&mut self, slot: UiExpressionSlot) -> Option<UiExpressionEvaluationRecord> {
        self.0.get_mut(slot.index())?.take()
    }

    /// Stores `record` in its own slot's cell. Returns `false`, storing
    /// nothing, when the slot is outside these records.
    pub(super) fn admit(&mut self, record: UiExpressionEvaluationRecord) -> bool {
        match self.0.get_mut(record.slot.index()) {
            Some(cell) => {
                *cell = Some(record);
                true
            }
            None => false,
        }
    }
}
