use super::super::*;

impl PoolState {
    pub(in crate::physical_residency::pool) fn append_evictable(
        &mut self,
        coordinate: RecordFrameCoordinate,
    ) {
        let slot = self
            .frames
            .slot_id_for_coordinate(coordinate)
            .expect("an evictable frame remains resident");
        let prior_tail = self.evictable_tail;
        let entry = self
            .frames
            .frame_at_slot_mut(slot)
            .expect("an evictable frame remains resident");
        entry.older_evictable = prior_tail;
        entry.newer_evictable = None;
        if let Some(tail) = prior_tail {
            self.frames
                .frame_at_slot_mut(tail)
                .expect("eviction tail remains resident")
                .newer_evictable = Some(slot);
        } else {
            self.evictable_head = Some(slot);
        }
        self.evictable_tail = Some(slot);
    }

    pub(in crate::physical_residency::pool) fn detach_evictable(
        &mut self,
        coordinate: RecordFrameCoordinate,
    ) {
        let Some(slot) = self.frames.slot_id_for_coordinate(coordinate) else {
            return;
        };
        self.detach_evictable_slot(slot);
    }

    pub(super) fn detach_evictable_slot(&mut self, slot: frame_table::FrameSlotId) {
        let entry = self
            .frames
            .frame_at_slot(slot)
            .expect("an eviction-order slot remains resident")
            .1;
        let older = entry.older_evictable;
        let newer = entry.newer_evictable;
        if older.is_none() && newer.is_none() && self.evictable_head != Some(slot) {
            return;
        }
        if let Some(older) = older {
            self.frames
                .frame_at_slot_mut(older)
                .expect("older eviction neighbor remains resident")
                .newer_evictable = newer;
        } else {
            self.evictable_head = newer;
        }
        if let Some(newer) = newer {
            self.frames
                .frame_at_slot_mut(newer)
                .expect("newer eviction neighbor remains resident")
                .older_evictable = older;
        } else {
            self.evictable_tail = older;
        }
        let entry = self
            .frames
            .frame_at_slot_mut(slot)
            .expect("detached eviction entry remains resident");
        entry.older_evictable = None;
        entry.newer_evictable = None;
    }
}
