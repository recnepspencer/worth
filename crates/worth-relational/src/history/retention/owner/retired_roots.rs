//! The original root inventory has one explicit gate, released before lane work.
use super::*;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

impl RelationalBranchRetentionOwnerInner {
    pub(in crate::history::retention) fn read_retired_roots(
        &self,
    ) -> RwLockReadGuard<'_, BTreeMap<usize, RelationalRetiredBranchRoot>> {
        self.retired_roots
            .read()
            .unwrap_or_else(|error| error.into_inner())
    }
    pub(in crate::history::retention) fn write_retired_roots(
        &self,
    ) -> RwLockWriteGuard<'_, BTreeMap<usize, RelationalRetiredBranchRoot>> {
        self.retired_roots
            .write()
            .unwrap_or_else(|error| error.into_inner())
    }
}

impl RelationalBranchRetentionOwner {
    /// Close retains only one actual immutable root. No queue, diagnostic
    /// inventory, liveness count or growing graph roster participates.
    pub(crate) fn next_source_retirement_root(
        &self,
        after: Option<usize>,
    ) -> Result<Option<(usize, Arc<RelationalBranchRoot>)>, ()> {
        use std::ops::Bound::{Excluded, Unbounded};
        let roots = match self.inner.retired_roots.try_read() {
            Ok(roots) => roots,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(()),
        };
        let next = match after {
            None => roots.first_key_value(),
            Some(after) => roots.range((Excluded(after), Unbounded)).next(),
        };
        Ok(next.map(|(&key, entry)| (key, entry.root.clone())))
    }
}
