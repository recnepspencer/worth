use super::DenseBitset;
use crate::data::persistent_vector::RetainedVectorMutationDenial;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

// A u64 has no nested allocation. This covers both measurements of one
// selected word and its fixed page bookkeeping, independent of bitset length.
const WORD_CHARGE_VISITS: usize = 32;

impl DenseBitset {
    pub(crate) fn prepared_retained_charge(&self) -> Result<Charge, RetainedVectorMutationDenial> {
        self.words.prepared_retained_charge()
    }

    pub(crate) fn prepare_retained_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        self.words.prepare_retained_charge(work)
    }

    pub(super) fn write_word(&mut self, index: usize, value: u64) {
        let mut work = Preparation::new(WORD_CHARGE_VISITS);
        if self
            .words
            .edit_with_retained_charge(index, &mut work, |word| *word = value)
            .is_err()
        {
            // This infallible storage operation also serves unprepared graph
            // construction. A pre-edit accounting denial preserves that
            // posture; it never manufactures readiness by scanning storage.
            self.words[index] = value;
        }
        // An Unaccounted result already performed the write and left its
        // charge unavailable. It must not cause the edit to be executed twice.
    }

    pub(super) fn append_zero_word(&mut self) {
        let mut work = Preparation::new(WORD_CHARGE_VISITS);
        if self.words.push_with_retained_charge(0, &mut work).is_err() {
            self.words.push_back(0);
        }
    }

    pub(super) fn charge_cleared_words(&mut self) {
        // clear_all already touches every word and iter_mut materializes
        // exclusive storage. Count that complete selected extent explicitly;
        // single-word mutation never pays this reconstruction/traversal cost.
        if let Some(visits) = self.words.len().checked_add(2) {
            let _ = self
                .words
                .prepare_retained_charge(&mut Preparation::new(visits));
        }
    }
}

impl RetainedStorageMeasurement for DenseBitset {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.words.retained_heap_charge(work)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for DenseBitset {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        self.words.prepare_fork_charge(work)
    }
}
