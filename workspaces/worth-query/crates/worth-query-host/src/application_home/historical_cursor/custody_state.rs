//! In-memory loan and in-flight retain custody tracking for retained historical
//! product branches on [`super::super::WorthQueryApplicationHome`].

use std::collections::{BTreeMap, BTreeSet};

use worth_query_execution::facade::product::WorthQueryProductBranch;

#[derive(Default)]
struct BranchCustodyEntry {
    active_loans: usize,
    prune_requested: bool,
    branch: Option<WorthQueryProductBranch>,
}

/// Coordinates active Query cursor `ReaderLoan`s with retained product-branch retirement.
#[derive(Default)]
pub(crate) struct RetainedBranchCustody {
    entries: BTreeMap<String, BranchCustodyEntry>,
    inflight_retains: BTreeSet<String>,
}

impl RetainedBranchCustody {
    pub(super) fn begin_inflight_retain(&mut self, branch_name: &str) {
        self.inflight_retains.insert(branch_name.to_owned());
    }

    pub(super) fn finish_inflight_retain(&mut self, branch_name: &str) {
        self.inflight_retains.remove(branch_name);
    }

    pub(super) fn is_inflight_retain(&self, branch_name: &str) -> bool {
        self.inflight_retains.contains(branch_name)
    }

    pub(super) fn acquire_loan(
        &mut self,
        branch_name: &str,
        branch: WorthQueryProductBranch,
    ) -> Result<(), ()> {
        let entry = self.entries.entry(branch_name.to_owned()).or_default();
        if entry.prune_requested {
            return Err(());
        }
        entry.active_loans = entry.active_loans.saturating_add(1);
        entry.branch = Some(branch);
        Ok(())
    }

    pub(super) fn request_prune(
        &mut self,
        branch_name: &str,
        resolved_branch: Option<WorthQueryProductBranch>,
    ) -> Option<WorthQueryProductBranch> {
        let entry = self.entries.entry(branch_name.to_owned()).or_default();
        entry.prune_requested = true;
        if entry.branch.is_none() {
            entry.branch = resolved_branch;
        }
        if entry.active_loans == 0 {
            self.entries
                .remove(branch_name)
                .and_then(|removed| removed.branch)
        } else {
            None
        }
    }

    pub(super) fn restore_pending_prune(
        &mut self,
        branch_name: &str,
        branch: WorthQueryProductBranch,
    ) {
        let entry = self.entries.entry(branch_name.to_owned()).or_default();
        entry.prune_requested = true;
        entry.branch = Some(branch);
    }

    pub(super) fn clear_retired(&mut self, branch_name: &str) {
        if let Some(entry) = self.entries.get(branch_name)
            && entry.active_loans == 0
        {
            self.entries.remove(branch_name);
        }
    }

    pub(super) fn release_loan(&mut self, branch_name: &str) -> Option<WorthQueryProductBranch> {
        let entry = self.entries.get_mut(branch_name)?;
        entry.active_loans = entry.active_loans.saturating_sub(1);
        if entry.active_loans > 0 {
            return None;
        }
        let removed = self.entries.remove(branch_name)?;
        removed.prune_requested.then_some(removed.branch).flatten()
    }

    pub(super) fn has_active_loans(&self, branch_name: &str) -> bool {
        self.entries
            .get(branch_name)
            .is_some_and(|e| e.active_loans > 0)
    }
}
