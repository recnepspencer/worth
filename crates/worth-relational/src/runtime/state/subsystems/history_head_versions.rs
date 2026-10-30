//! Exact live head bases and their fixed-depth oldest-version index.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::branch::RelationalBranchRoot;
use crate::identity::data::VersionId;
use crate::schema::data::SchemaVersionId;

type HeadBasis = (VersionId, SchemaVersionId);

const VERSION_BIT_DEPTH: usize = u64::BITS as usize;

#[derive(Clone, Debug)]
pub(super) struct BranchHeadVersionIndex {
    counts: HashMap<u64, u32>,
    bases: HashMap<HeadBasis, u32>,
    occupied_prefixes: Vec<HashSet<u64>>,
}

impl Default for BranchHeadVersionIndex {
    fn default() -> Self {
        Self {
            counts: HashMap::new(),
            bases: HashMap::new(),
            occupied_prefixes: (0..=VERSION_BIT_DEPTH).map(|_| HashSet::new()).collect(),
        }
    }
}

impl BranchHeadVersionIndex {
    pub(super) fn move_head(&mut self, previous: Option<HeadBasis>, next: Option<HeadBasis>) {
        if previous == next {
            return;
        }
        if let Some(previous) = previous {
            let count = self.bases.get_mut(&previous).expect(/* Proof: Only actual installed head movement supplies the previous basis, and installation registers that exact version/schema under this same mutex. */ "removed head basis is registered");
            *count -= 1;
            if *count == 0 {
                self.bases.remove(&previous);
            }
            self.remove(previous.0);
        }
        if let Some(next) = next {
            *self.bases.entry(next).or_default() += 1;
            self.insert(next.0);
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
    pub(crate) fn move_head(
        &self,
        previous: Option<&RelationalBranchRoot>,
        next: Option<&RelationalBranchRoot>,
    ) {
        self.index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .move_head(previous.and_then(head_basis), next.and_then(head_basis));
    }

    pub(crate) fn contains(&self, version: VersionId, schema: SchemaVersionId) -> bool {
        self.index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .bases
            .contains_key(&(version, schema))
    }

    /// Replace the whole index with the given live heads under one lock, so a
    /// concurrent fence read never observes a half-rebuilt index.
    pub(crate) fn rebuild(&self, roots: impl IntoIterator<Item = Arc<RelationalBranchRoot>>) {
        let mut index = self
            .index
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *index = BranchHeadVersionIndex::default();
        for root in roots {
            index.move_head(None, head_basis(&root));
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

fn head_basis(root: &RelationalBranchRoot) -> Option<HeadBasis> {
    Some((
        root.canonical_envelope()?.commit.version_id,
        root.schema_authority().schema_version(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oldest_head_is_exact_across_shared_versions_and_forward_movement() {
        let mut index = BranchHeadVersionIndex::default();
        let basis = |version| Some((VersionId(version), SchemaVersionId(1)));
        index.move_head(None, basis(9));
        index.move_head(None, basis(3));
        index.move_head(None, basis(3));
        assert_eq!(index.oldest(), Some(VersionId(3)));
        index.move_head(basis(3), basis(12));
        assert_eq!(index.oldest(), Some(VersionId(3)));
        index.move_head(basis(3), None);
        assert_eq!(index.oldest(), Some(VersionId(9)));
    }
}
