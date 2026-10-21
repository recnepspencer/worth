//! Bit-exact model facts and eligibility, independent of runtime records.
use super::*;

impl PartialEq for ModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.binding == other.binding
            && self.region == other.region
            && self.incoming == other.incoming
            && self.value.to_bits() == other.value.to_bits()
            && self.work == other.work
            && self.fault == other.fault
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.odd == other.odd
            && self.weights.map(f64::to_bits) == other.weights.map(f64::to_bits)
    }
}

impl Model {
    pub(in super::super::super) fn signed_zero(rng: &mut Lcg) -> Self {
        let mut model = Self::bounded(rng, 1, 3, TOTALS_WORK);
        let entry = model.entries[0].as_mut().unwrap();
        entry.incoming = false;
        entry.region = 1;
        entry.work = 1;
        entry.value = -0.0;
        model.weights = [-0.0; 2];
        model
    }

    pub(in super::super::super) fn completes(&self) -> bool {
        self.completes_at(TOTALS_WORK)
    }
    pub(in super::super::super) fn completes_at(&self, ceiling: usize) -> bool {
        !self.entries.iter().flatten().any(|entry| entry.fault)
            && self
                .entries
                .iter()
                .flatten()
                .map(|entry| entry.work)
                .sum::<u64>()
                + self
                    .regions()
                    .iter()
                    .filter(|key| key.is_multiple_of(2))
                    .count() as u64
                <= ceiling as u64
    }
}
