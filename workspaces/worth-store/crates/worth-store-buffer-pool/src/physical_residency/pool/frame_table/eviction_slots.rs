use super::*;

impl FrameTable {
    /// Resolve an indexed exact coordinate to its stable occupied slot.
    /// A bounded resident has the same slot in both lookup indexes.
    pub(in crate::physical_residency::pool) fn slot_id_for_coordinate(
        &self,
        coordinate: RecordFrameCoordinate,
    ) -> Option<FrameSlotId> {
        let id = *self.exact_index.get(&coordinate)?;
        self.frame_at_slot(id)
            .filter(|(actual, _)| *actual == coordinate)
            .map(|_| id)
    }

    pub(in crate::physical_residency::pool) fn frame_at_slot(
        &self,
        id: FrameSlotId,
    ) -> Option<(RecordFrameCoordinate, &FrameEntry)> {
        match self.slots.get(id.0 as usize)?.as_ref()? {
            FrameSlot::Exact { coordinate, frame } => Some((*coordinate, frame)),
            FrameSlot::Bounded { artifact, entry } => entry
                .resident_coordinate_for_artifact(*artifact)
                .zip(entry.resident_frame()),
        }
    }

    pub(in crate::physical_residency::pool) fn frame_at_slot_mut(
        &mut self,
        id: FrameSlotId,
    ) -> Option<&mut FrameEntry> {
        match self.slots.get_mut(id.0 as usize)?.as_mut()? {
            FrameSlot::Exact { frame, .. } => Some(frame),
            FrameSlot::Bounded { entry, .. } => entry.resident_frame_mut(),
        }
    }
}
