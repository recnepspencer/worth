//! Fixed-depth index for the oldest authoritative live branch-head version.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::identity::data::VersionId;

const VERSION_BIT_DEPTH: usize = u64::BITS as usize;

#[derive(Clone, Debug)]
pub(super) struct BranchHeadVersionIndex {
    counts: HashMap<u64, u32>,
    occupied_prefixes: Vec<HashSet<u64>>,
}

impl Default for BranchHeadVersionIndex {
    fn default() -> Self {
        Self {
            counts: HashMap::new(),
            occupied_prefixes: (0..=VERSION_BIT_DEPTH).map(|_| HashSet::new()).collect(),
        }
    }
}

impl BranchHeadVersionIndex {
    pub(super) fn move_head(&mut self, previous: Option<VersionId>, next: Option<VersionId>) {
        if previous == next {
            return;
        }
        if let Some(previous) = previous {
            self.remove(previous);
        }
        if let Some(next) = next {
            self.insert(next);
        }
    }

    pub(super) fn oldest(&self) -> Option<VersionId> {
        if self.counts.is_empty() {
            return None;
        }
        let mut prefix = 0_u64;
        for lower_bits in (0..VERSION_BIT_DEPTH).rev() {
            let zero_child = prefix << 1;
            prefix = if self.occupied_prefixes[lower_bits].contains(&zero_child) {
                zero_child
            } else {
                zero_child | 1
            };
        }
        Some(VersionId(prefix))
    }

    fn insert(&mut self, version: VersionId) {
        let count = self.counts.entry(version.0).or_default();
        *count += 1;
        if *count != 1 {
            return;
        }
        for lower_bits in 0..=VERSION_BIT_DEPTH {
            self.occupied_prefixes[lower_bits]
                .insert(version.0.checked_shr(lower_bits as u32).unwrap_or(0));
        }
    }

    fn remove(&mut self, version: VersionId) {
        let count = self
            .counts
            .get_mut(&version.0)
            .expect(/* Proof: Head installation increments this exact version index and removal consumes the same registered head once under exclusive owner mutation. */ "removed branch head must be present in the authoritative version index");
        *count -= 1;
        if *count != 0 {
            return;
        }
        self.counts.remove(&version.0);
        self.occupied_prefixes[0].remove(&version.0);
        for lower_bits in 1..=VERSION_BIT_DEPTH {
            let prefix = version.0.checked_shr(lower_bits as u32).unwrap_or(0);
            let child = prefix << 1;
            if self.occupied_prefixes[lower_bits - 1].contains(&child)
                || self.occupied_prefixes[lower_bits - 1].contains(&(child | 1))
            {
                break;
            }
            self.occupied_prefixes[lower_bits].remove(&prefix);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BranchHeadVersionIndexAuthority {
    index: Arc<Mutex<BranchHeadVersionIndex>>,
}

impl BranchHeadVersionIndexAuthority {
    pub(crate) fn move_head(&self, previous: Option<VersionId>, next: Option<VersionId>) {
        self.index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .move_head(previous, next);
    }

    /// Replace the whole index with the given live heads under one lock, so a
    /// concurrent fence read never observes a half-rebuilt index.
    pub(crate) fn rebuild(&self, versions: impl IntoIterator<Item = VersionId>) {
        let mut index = self
            .index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *index = BranchHeadVersionIndex::default();
        for version in versions {
            index.move_head(None, Some(version));
        }
    }

    pub(super) fn oldest(&self) -> Option<VersionId> {
        self.index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .oldest()
    }

    pub(super) fn detached(&self) -> Self {
        let index = self
            .index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        Self {
            index: Arc::new(Mutex::new(index)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oldest_head_is_exact_across_shared_versions_and_forward_movement() {
        let mut index = BranchHeadVersionIndex::default();
        index.move_head(None, Some(VersionId(9)));
        index.move_head(None, Some(VersionId(3)));
        index.move_head(None, Some(VersionId(3)));
        assert_eq!(index.oldest(), Some(VersionId(3)));
        index.move_head(Some(VersionId(3)), Some(VersionId(12)));
        assert_eq!(index.oldest(), Some(VersionId(3)));
        index.move_head(Some(VersionId(3)), None);
        assert_eq!(index.oldest(), Some(VersionId(9)));
    }
}
