//! The original root inventory has one explicit gate, released before lane work.
use super::*;
use crate::history::retention::RetentionBusy;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use worth_foundational::DagDirtyList;

#[derive(Debug, Default)]
pub(in crate::internal_port::history::retention) struct RetiredBranchRoots {
    roots: BTreeMap<usize, RelationalRetiredBranchRoot>,
    dirty: DagDirtyList<usize>,
}

impl Deref for RetiredBranchRoots {
    type Target = BTreeMap<usize, RelationalRetiredBranchRoot>;

    fn deref(&self) -> &Self::Target {
        &self.roots
    }
}

impl DerefMut for RetiredBranchRoots {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.roots
    }
}

impl RetiredBranchRoots {
    pub(in crate::internal_port::history::retention) fn changed_count(&self) -> usize {
        self.dirty.len()
    }

    pub(in crate::internal_port::history::retention) fn mark_changed(&mut self, key: usize) {
        let mut links = self.roots[&key].dirty;
        self.dirty.mark(key, &mut links, |neighbor, edit| {
            self.roots
                .get_mut(&neighbor)
                .expect("live dirty neighbor")
                .dirty
                .apply(edit);
        });
        self.roots.get_mut(&key).expect("changed root").dirty = links;
    }

    pub(in crate::internal_port::history::retention) fn pop_changed(&mut self) -> Option<usize> {
        let key = self.dirty.head()?;
        let mut links = std::mem::take(&mut self.roots.get_mut(&key).expect("dirty root").dirty);
        self.dirty.forget(&mut links, |neighbor, edit| {
            self.roots
                .get_mut(&neighbor)
                .expect("live dirty neighbor")
                .dirty
                .apply(edit);
        });
        Some(key)
    }
}

impl RelationalBranchRetentionOwnerInner {
    pub(in crate::internal_port::history::retention) fn read_retired_roots(
        &self,
    ) -> RwLockReadGuard<'_, RetiredBranchRoots> {
        self.retired_roots
            .read()
            .unwrap_or_else(|error| error.into_inner())
    }
    pub(in crate::internal_port::history::retention) fn write_retired_roots(
        &self,
    ) -> RwLockWriteGuard<'_, RetiredBranchRoots> {
        self.retired_roots
            .write()
            .unwrap_or_else(|error| error.into_inner())
    }
}

impl RelationalBranchRetentionOwner {
    /// Close retains only one actual immutable root. No queue, diagnostic
    /// inventory, liveness count or growing graph roster participates.
    pub fn next_source_retirement_root(
        &self,
        after: Option<usize>,
    ) -> Result<Option<(usize, Arc<RelationalBranchRoot>)>, RetentionBusy> {
        use std::ops::Bound::{Excluded, Unbounded};
        let roots = match self.inner.retired_roots.try_read() {
            Ok(roots) => roots,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(RetentionBusy),
        };
        let next = match after {
            None => roots.first_key_value(),
            Some(after) => roots.range((Excluded(after), Unbounded)).next(),
        };
        Ok(next.map(|(&key, entry)| (key, entry.root.clone())))
    }
}
