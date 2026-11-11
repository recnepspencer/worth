use crate::storage::substrate::SharedMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DenseSlotBitSet {
    words: SharedMap<usize, u64>,
    population: usize,
}

impl DenseSlotBitSet {
    /// Borrow canonical nonzero words; full serialization creates no scratch Vec.
    pub(crate) fn words(&self) -> impl ExactSizeIterator<Item = (usize, u64)> + '_ {
        self.words.iter().map(|(&index, &word)| (index, word))
    }
    pub(crate) fn stored_word_count(&self) -> usize {
        self.words.len()
    }
    pub(crate) fn changed_words_since(&self, previous: Option<&Self>) -> Vec<(usize, Option<u64>)> {
        let empty = SharedMap::default();
        let previous = previous.map(|set| &set.words).unwrap_or(&empty);
        self.words
            .changed_keys_since(previous)
            .into_iter()
            .map(|key| (key, self.words.get(&key).copied()))
            .collect()
    }

    pub(crate) fn visit_new_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        self.words.visit_new_allocations(
            &previous.words,
            visitor,
            &mut crate::storage::substrate::visit_new_inline_value,
        );
    }
    pub(crate) fn with_capacity(_capacity: usize) -> Self {
        Self {
            words: SharedMap::new(),
            population: 0,
        }
    }

    pub(crate) fn set_true_allocation_bytes(&self, slot: usize) -> Option<u64> {
        let word = slot / 64;
        let previous = self.words.get(&word).copied().unwrap_or(0);
        if previous & (1 << (slot % 64)) != 0 {
            return Some(0);
        }
        self.words.insertion_allocation_bytes(&word)
    }

    pub(crate) fn owned_allocation_bytes(&self) -> Option<u64> {
        self.words.shallow_allocation_bytes()
    }

    pub(crate) fn try_set_slots<Stop>(
        &self,
        prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<DenseSlotIterator<'_>, Stop> {
        Ok(DenseSlotIterator {
            words: self.words.try_iter(prepare)?,
            word_index: 0,
            remaining: 0,
        })
    }

    pub(crate) fn set_for_request(
        &mut self,
        slot: usize,
        value: bool,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<u64, crate::execution::RelationalExecutionDenialCause> {
        let (word, previous, next) = self.changed_word(slot, value);
        let changed = if next == previous {
            Ok(0)
        } else if next == 0 {
            self.words
                .remove_for_request(&word, request)
                .map(|(_, bytes)| bytes)
        } else {
            self.words.insert_for_request(word, next, request)
        }?;
        self.population =
            self.population - previous.count_ones() as usize + next.count_ones() as usize;
        Ok(changed)
    }
    fn changed_word(&self, slot: usize, value: bool) -> (usize, u64, u64) {
        let word = slot / 64;
        let previous = self.words.get(&word).copied().unwrap_or(0);
        let next = if value {
            previous | (1 << (slot % 64))
        } else {
            previous & !(1 << (slot % 64))
        };
        (word, previous, next)
    }

    pub(crate) fn set(&mut self, slot: usize, value: bool) -> u64 {
        let (word, previous, next) = self.changed_word(slot, value);
        self.population =
            self.population - previous.count_ones() as usize + next.count_ones() as usize;
        if next == previous {
            0
        } else if next == 0 {
            self.words.remove(&word).1
        } else {
            self.words.insert(word, next)
        }
    }

    pub(crate) fn count_ones(&self) -> usize {
        self.population
    }

    pub(crate) fn count_ones_in_range(&self, start: usize, end: usize) -> usize {
        if start >= end {
            return 0;
        }
        let start_word = start / 64;
        let end_word = (end - 1) / 64;
        let start_bit = start % 64;
        let end_bit = (end - 1) % 64;
        let mut total = 0usize;

        if start_word == end_word {
            let Some(word) = self.words.get(&start_word).copied() else {
                return 0;
            };
            let lower_mask = (!0u64) << start_bit;
            let upper_mask = if end_bit == 63 {
                !0u64
            } else {
                (1u64 << (end_bit + 1)) - 1
            };
            return (word & lower_mask & upper_mask).count_ones() as usize;
        }

        if let Some(word) = self.words.get(&start_word).copied() {
            total += (word & ((!0u64) << start_bit)).count_ones() as usize;
        }

        for word_index in (start_word + 1)..end_word {
            total += self
                .words
                .get(&word_index)
                .copied()
                .unwrap_or(0)
                .count_ones() as usize;
        }

        if let Some(word) = self.words.get(&end_word).copied() {
            let upper_mask = if end_bit == 63 {
                !0u64
            } else {
                (1u64 << (end_bit + 1)) - 1
            };
            total += (word & upper_mask).count_ones() as usize;
        }

        total
    }

    pub(crate) fn iter_set_slots(&self) -> Vec<usize> {
        self.set_slots().collect()
    }

    /// Iterates set slots in canonical order without materializing their IDs.
    pub(crate) fn set_slots(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().flat_map(|(&word_index, &word)| {
            let mut remaining = word;
            std::iter::from_fn(move || {
                if remaining == 0 {
                    return None;
                }
                let bit = remaining.trailing_zeros() as usize;
                remaining &= remaining - 1;
                Some(word_index * 64 + bit)
            })
        })
    }

    pub(crate) fn from_words(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        words: Vec<u64>,
    ) -> Self {
        let population = words.iter().map(|word| word.count_ones() as usize).sum();
        Self {
            population,
            words: words
                .into_iter()
                .enumerate()
                .filter(|(_, word)| *word != 0)
                .collect(),
        }
    }

    pub(crate) fn from_sparse_words(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        words: Vec<(u64, u64)>,
    ) -> Self {
        let words: SharedMap<usize, u64> = words
            .into_iter()
            .filter(|(_, word)| *word != 0)
            .map(|(index, word)| (index as usize, word))
            .collect();
        // Sparse reconstruction preserves the last nonzero value at each key.
        let population = words.values().map(|word| word.count_ones() as usize).sum();
        Self { words, population }
    }

    pub(crate) fn from_words_for_request(
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        words: impl IntoIterator<Item = (usize, u64)>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        let mut result = Self::with_capacity(0);
        for (index, word) in words {
            request.consult()?;
            if word != 0 {
                let previous = result.words.get(&index).copied().unwrap_or(0);
                result.words.insert_for_request(index, word, request)?;
                result.population =
                    result.population - previous.count_ones() as usize + word.count_ones() as usize;
            }
        }
        Ok(result)
    }

    pub(crate) fn sparse_words(&self) -> Vec<(u64, u64)> {
        self.words
            .iter()
            .map(|(&index, &word)| (index as u64, word))
            .collect()
    }

    pub(crate) fn represented_slot_capacity(&self) -> usize {
        self.words
            .last_key_value()
            .map_or(0, |(&word, _)| (word + 1).saturating_mul(64))
    }

    pub(crate) fn visit_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        self.words.visit_allocations(
            unique,
            visitor,
            &mut crate::storage::substrate::visit_inline_value,
        );
    }
}

/// A borrowed word walk with its addressably bounded inline AVL stack.
pub(crate) struct DenseSlotIterator<'a> {
    words: crate::storage::substrate::SharedMapIter<'a, usize, u64>,
    word_index: usize,
    remaining: u64,
}
impl Iterator for DenseSlotIterator<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.remaining == 0 {
            let (&index, &word) = self.words.next()?;
            self.word_index = index;
            self.remaining = word;
        }
        let bit = self.remaining.trailing_zeros() as usize;
        self.remaining &= self.remaining - 1;
        Some(self.word_index * 64 + bit)
    }
}

#[cfg(test)]
mod population_tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn carried_population_matches_independent_slot_set_across_edits_and_clones() {
        let mut bits = DenseSlotBitSet::with_capacity(0);
        let mut selected = BTreeSet::new();
        let mut seed = 17_u64;
        for step in 0..4096 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let slot = ((seed >> 32) % 4096) as usize;
            let value = seed & 16 != 0;
            let previous = bits.clone();
            let old = selected.clone();
            bits.set(slot, value);
            if value {
                selected.insert(slot);
            } else {
                selected.remove(&slot);
            }
            assert_eq!(bits.count_ones(), selected.len(), "step {step}");
            assert_eq!(previous.count_ones(), old.len());
            assert_eq!(bits.set_slots().collect::<BTreeSet<_>>(), selected);
        }
        for rebuilt in [
            DenseSlotBitSet::from_words(
                &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                vec![u64::MAX, 0, 5],
            ),
            DenseSlotBitSet::from_sparse_words(
                &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                vec![(4096, 7), (4096, 1), (99, 0)],
            ),
        ] {
            assert_eq!(
                rebuilt.count_ones(),
                rebuilt.set_slots().collect::<BTreeSet<_>>().len()
            );
        }
    }
}
