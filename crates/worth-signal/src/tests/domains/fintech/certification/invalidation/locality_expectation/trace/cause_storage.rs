use super::{ExpectedTrace, LocalitySemanticOutputId};

impl ExpectedTrace {
    pub(super) fn stage_pre_rewire_work(&mut self, target: LocalitySemanticOutputId) {
        self.allocate_readiness_epoch();
        self.settle_pending_cause(target);
    }

    pub(super) fn admit_pending_cause(&mut self, target: LocalitySemanticOutputId) {
        if self.pending_cause_slots.contains_key(&target) {
            return;
        }
        let index = match self.free_cause_slots.pop() {
            Some(index) => index,
            None => {
                self.cause_slot_generations
                    .push(self.cause_store_generation);
                (self.cause_slot_generations.len() - 1) as u32
            }
        };
        let generation = u64::from(self.cause_slot_generations[index as usize]);
        self.pending_cause_slots.insert(target, (index, generation));
    }

    pub(super) fn pending_cause_generation(&self, target: LocalitySemanticOutputId) -> u64 {
        self.pending_cause_slots
            .get(&target)
            .unwrap_or_else(|| {
                panic!(
                    "expected dependency target {target:?} lacks a pending cause; live={:?}",
                    self.pending_cause_slots.keys().collect::<Vec<_>>()
                )
            })
            .1
    }

    pub(super) fn has_pending_cause(&self, target: LocalitySemanticOutputId) -> bool {
        self.pending_cause_slots.contains_key(&target)
    }

    pub(super) fn settle_pending_cause(&mut self, target: LocalitySemanticOutputId) {
        let (index, _) = self
            .pending_cause_slots
            .remove(&target)
            .expect("settled dependency work must own a pending cause slot");
        self.cause_slot_generations[index as usize] = self.cause_slot_generations[index as usize]
            .checked_add(1)
            .expect("expected cause-set generation overflow");
        self.free_cause_slots.push(index);
    }
}
