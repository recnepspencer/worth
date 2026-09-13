mod reserved_fork;
mod retained_charge;
#[cfg(test)]
mod retained_charge_tests;

/// Dense bitset for deterministic, allocation-light index marking.
#[derive(Debug, Clone, Default)]
pub struct DenseBitset {
    words: crate::data::persistent_vector::PersistentVector<u64>,
}

pub struct DenseBitsetIter<'a> {
    words: &'a crate::data::persistent_vector::PersistentVector<u64>,
    word_index: usize,
    pending: u64,
}

impl DenseBitset {
    /// Create an empty bitset.
    pub fn new() -> Self {
        Self {
            words: crate::data::persistent_vector::PersistentVector::new(),
        }
    }

    /// Ensure capacity for indices up to `len`.
    pub fn ensure_len(&mut self, len: usize) {
        let word_len = len.div_ceil(64);
        if self.words.len() < word_len {
            let missing = word_len - self.words.len();
            for _ in 0..missing {
                self.append_zero_word();
            }
        }
    }

    /// Mark one index. Returns true if this call changed the bit.
    pub fn mark(&mut self, idx: usize) -> bool {
        let word_idx = idx / 64;
        if word_idx >= self.words.len() {
            let missing = word_idx + 1 - self.words.len();
            for _ in 0..missing {
                self.append_zero_word();
            }
        }
        let bit = 1u64 << (idx % 64);
        let before = self.words[word_idx];
        self.write_word(word_idx, before | bit);
        before != self.words[word_idx]
    }

    /// Clear one index.
    pub fn clear(&mut self, idx: usize) {
        let word_idx = idx / 64;
        if word_idx >= self.words.len() {
            return;
        }
        let bit = 1u64 << (idx % 64);
        self.write_word(word_idx, self.words[word_idx] & !bit);
    }

    /// Return whether index is marked.
    pub fn contains(&self, idx: usize) -> bool {
        let word_idx = idx / 64;
        if word_idx >= self.words.len() {
            return false;
        }
        let bit = 1u64 << (idx % 64);
        (self.words[word_idx] & bit) != 0
    }

    /// Clear all bits.
    pub fn clear_all(&mut self) {
        for word in self.words.iter_mut() {
            *word = 0;
        }
        self.charge_cleared_words();
    }

    /// Merge `other` into this bitset.
    #[cfg(test)]
    pub fn merge(&mut self, other: &Self) {
        if self.words.len() < other.words.len() {
            let missing = other.words.len() - self.words.len();
            for _ in 0..missing {
                self.append_zero_word();
            }
        }
        for (idx, word) in other.words.iter().copied().enumerate() {
            self.write_word(idx, self.words[idx] | word);
        }
    }

    /// Iterate marked indices in ascending deterministic order.
    pub fn iter_marked(&self) -> DenseBitsetIter<'_> {
        DenseBitsetIter {
            words: &self.words,
            word_index: 0,
            pending: 0,
        }
    }

    /// Return marked indices in ascending order.
    pub fn marked_indices(&self) -> Vec<usize> {
        self.iter_marked().collect()
    }

    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            words: self.words.operational_clone(),
        }
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            words: self.words.fork_persistent(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.words.shares_storage_with(&other.words)
    }
}

impl Iterator for DenseBitsetIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pending != 0 {
                let bit = self.pending.trailing_zeros() as usize;
                self.pending &= self.pending - 1;
                return Some((self.word_index - 1) * 64 + bit);
            }
            let word = *self.words.get(self.word_index)?;
            self.word_index += 1;
            if word != 0 {
                self.pending = word;
            }
        }
    }
}
