//! Borrowed occupied slots with cancellation checkpoints before iterator descent.
use super::{RecordArena, RecordKind, SlotView};

pub(crate) enum RecordSlotVisit<'a, K: RecordKind> {
    Navigation(u64),
    Slot {
        logical: usize,
        view: SlotView<'a, K>,
    },
}

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn try_visit_occupied<Stop>(
        &self,
        mut visit: impl FnMut(RecordSlotVisit<'_, K>) -> Result<(), Stop>,
    ) -> Result<(), Stop> {
        let mut slots = self
            .slots
            .try_occupied_entries(|work, _| visit(RecordSlotVisit::Navigation(work)))?;
        while let Some((&logical, &physical)) =
            slots.try_next(|work, _| visit(RecordSlotVisit::Navigation(work)))?
        {
            visit(RecordSlotVisit::Slot {
                logical: logical as usize,
                view: SlotView::new(self, physical),
            })?;
        }
        Ok(())
    }
}
