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
        !self.entries.iter().flatten().any(|entry| entry.fault)
    }
}
