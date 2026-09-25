use super::{PinClass, RecordArena, RecordKind};

#[cfg(test)]
mod tests;

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

    pub(crate) fn adjust_named_pin(
        &mut self,
        slot: usize,
        class: PinClass,
        delta: i32,
    ) -> Option<()> {
        let physical = self.physical_index(slot)?;
        match class {
            #[cfg(test)]
            PinClass::Branch => self.branch_pins.adjust(physical, delta),
            PinClass::Replay => self.replay_pins.adjust(physical, delta),
        }
    }

    pub(crate) fn clear_all_pins(&mut self) {
        self.snapshot_pins.fill(0);
        self.branch_pins.fill(0);
        self.replay_pins.fill(0);
    }

    pub(crate) fn preserve_runtime_pins_from(&mut self, current: &Self) {
        let slots = &self.slots;
        let generations = &self.generations;
        let remap = |source| {
            let logical = current.slots.logical_slot(source)?;
            let physical = slots.physical_index(logical)?;
            let same_generation = generations[physical] == current.generations[source];
            debug_assert!(same_generation, "a pinned generation cannot be reused");
            same_generation.then_some(physical)
        };
        self.snapshot_pins.remap_from(&current.snapshot_pins, remap);
        self.branch_pins.remap_from(&current.branch_pins, remap);
        self.replay_pins.remap_from(&current.replay_pins, remap);
    }

    #[cfg(test)]
    pub(crate) fn clear_named_pins(&mut self, class: PinClass) {
        match class {
            PinClass::Branch => self.branch_pins.fill(0),
            PinClass::Replay => self.replay_pins.fill(0),
        }
    }
}
