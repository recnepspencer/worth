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
    pub(in super::super::super) fn same_decision_facts(
        &self,
        other: &Self,
        observation: bool,
    ) -> bool {
        if self.odd != other.odd || self.entries != other.entries {
            return false;
        }
        let keys = if observation {
            self.observation_keys()
        } else {
            self.clone()
        };
        // Only even gathered partitions declare a read of the selected set's
        // weight. Other set weights and an all-odd partition set are absent
        // from the producer's decision basis.
        !keys.regions().iter().any(|key| key.is_multiple_of(2))
            || self.weights[usize::from(self.odd)].to_bits()
                == other.weights[usize::from(other.odd)].to_bits()
    }

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
    /// Each declared adjacency retains edges + endpoint reads + two list units;
    /// reobservation adds one anchor read. Field reads retain one unit each.
    /// Shared outgoing lists are counted once, despite repeated key reads.
    pub(in super::super::super) fn comparison_bound(&self, observation: bool) -> usize {
        let items = self.len();
        let weight = usize::from(self.regions().iter().any(|key| key.is_multiple_of(2)));
        if !observation {
            return 7 * items + 3 + weight;
        }
        let incoming = self
            .entries
            .iter()
            .flatten()
            .filter(|entry| entry.incoming)
            .count();
        16 * items
            + 6
            + weight
            + if incoming == 0 {
                0
            } else {
                (2 * incoming + 5) * OBSERVATION_SETS
            }
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
