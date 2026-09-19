use super::{PinClass, RecordArena, RecordKind};

impl<K: RecordKind> RecordArena<K> {
    pub(crate) fn snapshot_pin_count(&self, slot: usize) -> Option<u32> {
        self.physical_index(slot)
            .and_then(|physical| self.snapshot_pins.get(physical).copied())
    }

    pub(crate) fn branch_pin_count(&self, slot: usize) -> Option<u32> {
        self.physical_index(slot)
            .and_then(|physical| self.branch_pins.get(physical).copied())
    }

    pub(crate) fn replay_pin_count(&self, slot: usize) -> Option<u32> {
        self.physical_index(slot)
            .and_then(|physical| self.replay_pins.get(physical).copied())
    }

    pub(crate) fn adjust_named_pin(&mut self, slot: usize, class: PinClass) -> Option<&mut u32> {
        let physical = self.physical_index(slot)?;
        match class {
            #[cfg(test)]
            PinClass::Branch => self.branch_pins.get_mut(physical),
            PinClass::Replay => self.replay_pins.get_mut(physical),
        }
    }

    pub(crate) fn clear_all_pins(&mut self) {
        self.snapshot_pins.fill(0);
        self.branch_pins.fill(0);
        self.replay_pins.fill(0);
    }

    pub(crate) fn preserve_runtime_pins_from(&mut self, current: &Self) {
        self.snapshot_pins.fill(0);
        self.branch_pins.fill(0);
        self.replay_pins.fill(0);
        for logical in self.occupied_slots() {
            let physical = self
                .physical_index(logical)
                .expect("occupied slot has a physical row");
            let Some(current_physical) = current.physical_index(logical) else {
                continue;
            };
            if self.generations[physical] != current.generations[current_physical] {
                debug_assert_eq!(current.snapshot_pins[current_physical], 0);
                debug_assert_eq!(current.branch_pins[current_physical], 0);
                debug_assert_eq!(current.replay_pins[current_physical], 0);
                continue;
            }
            self.snapshot_pins[physical] = current.snapshot_pins[current_physical];
            self.branch_pins[physical] = current.branch_pins[current_physical];
            self.replay_pins[physical] = current.replay_pins[current_physical];
        }
    }

    #[cfg(test)]
    pub(crate) fn clear_named_pins(&mut self, class: PinClass) {
        match class {
            PinClass::Branch => self.branch_pins.fill(0),
            PinClass::Replay => self.replay_pins.fill(0),
        }
    }
}
