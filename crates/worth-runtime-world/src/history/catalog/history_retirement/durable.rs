//! Two-phase durable history retirement and reopen disposal.
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::identity::CompositeCommitIdentity;

use super::super::support::{effective_parent, lock_state, remove_installed, validate_owner};
use super::super::{lock_index, CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::super::CompositeHistoryCatalogState;
use super::interior::EligibleInterior;
use super::{installed, installed_ancestry, RetiredBranchNeed, RetiredHistoryEntries};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DurableRetirementMode {
    Interior,
    RetiredBranch,
}

/// Reserves candidate commits in `state.retiring` across durable `HistoryRetired`
/// journal publication so no concurrent protection or child can race the journal.
pub(crate) struct PendingDurableHistoryRetirement {
    state: Arc<Mutex<CompositeHistoryCatalogState>>,
    commits: Vec<CompositeCommitIdentity>,
    cursor: usize,
    mode: DurableRetirementMode,
    held: Option<CompositeCommitIdentity>,
    armed: bool,
}

impl PendingDurableHistoryRetirement {
    pub(crate) fn remaining(&self) -> &[CompositeCommitIdentity] {
        &self.commits[self.cursor..]
    }

    /// Apply and remove the next `count` durably acknowledged commits from live
    /// history. Because `state.retiring` fences all protection and child
    /// reservation paths, every acknowledged commit must still satisfy its
    /// retirement invariant — silently skipping an acknowledged tombstone is
    /// forbidden.
    pub(crate) fn commit_next(&mut self, count: usize) -> RetiredHistoryEntries {
        assert!(
            self.cursor.saturating_add(count) <= self.commits.len(),
            "committed frame exceeds reserved durable retirement candidates"
        );
        let slice = &self.commits[self.cursor..self.cursor + count];
        let mut state = lock_state(&self.state);
        let mut retired = Vec::with_capacity(count);
        match self.mode {
            DurableRetirementMode::Interior => {
                for commit in slice {
                    assert!(
                        state.retiring.remove(commit),
                        "acknowledged interior commit must remain fenced until commit"
                    );
                    if let Some(eligible) = EligibleInterior::assess(&state, commit) {
                        retired.push(eligible.retire(&mut state));
                    } else {
                        // A concurrent retired-branch release may have removed
                        // this commit's sole child while fenced; in that case
                        // it has zero dependencies and zero protections and
                        // must be removed directly rather than skipped.
                        let reachability = lock_index(&state.reachability)
                            .lookup(commit)
                            .expect("acknowledged interior commit must remain installed");
                        assert!(
                            reachability.direct_protections() == 0
                                && reachability.descendant_dependencies() == 0
                                && !reachability.held_by_product_head(),
                            "acknowledged durable interior retirement violated retiring fence for {commit:?}: {reachability:?}"
                        );
                        retired.push(remove_installed(&mut state, commit));
                    }
                }
            }
            DurableRetirementMode::RetiredBranch => {
                for commit in slice {
                    assert!(
                        state.retiring.remove(commit),
                        "acknowledged retired-branch commit must remain fenced until commit"
                    );
                    let reachability = lock_index(&state.reachability)
                        .lookup(commit)
                        .expect("acknowledged retired-branch commit must remain installed");
                    assert!(
                        reachability.direct_protections() == 0
                            && reachability.descendant_dependencies() == 0
                            && !reachability.held_by_product_head(),
                        "acknowledged durable retired-branch retirement violated retiring fence for {commit:?}: {reachability:?}"
                    );
                    retired.push(remove_installed(&mut state, commit));
                }
            }
        }
        self.cursor += count;
        if self.cursor == self.commits.len() {
            self.armed = false;
        }
        retired
    }

    /// Release all uncommitted commits from `state.retiring` when the current
    /// frame failed with proven no effect (`Busy`, `Refused`, or preparation
    /// error), making them immediately eligible for retry.
    pub(crate) fn release_remaining_proven_no_effect(mut self) {
        self.release_range(self.cursor..self.commits.len());
        self.cursor = self.commits.len();
        self.armed = false;
    }

    /// Keep ONLY the indeterminate frame (`cursor..cursor + indeterminate_count`)
    /// fenced in `state.retiring`, and release the unsubmitted suffix
    /// (`cursor + indeterminate_count..`) so later passes can retry it.
    pub(crate) fn fence_indeterminate_and_release_suffix(mut self, indeterminate_count: usize) {
        let suffix_start = self
            .cursor
            .saturating_add(indeterminate_count)
            .min(self.commits.len());
        self.release_range(suffix_start..self.commits.len());
        self.armed = false;
    }

    pub(crate) fn finish_all(mut self) -> Option<CompositeCommitIdentity> {
        assert_eq!(
            self.cursor,
            self.commits.len(),
            "finish_all requires every reserved commit to be acknowledged"
        );
        self.armed = false;
        self.held.take()
    }

    fn release_range(&self, range: std::ops::Range<usize>) {
        if range.is_empty() {
            return;
        }
        let mut state = lock_state(&self.state);
        for commit in &self.commits[range] {
            state.retiring.remove(commit);
            if self.mode == DurableRetirementMode::Interior && installed(&state, commit).is_some() {
                lock_index(&state.reachability).mark_changed(commit);
            }
        }
    }
}

impl Drop for PendingDurableHistoryRetirement {
    fn drop(&mut self) {
        // When dropped without explicit settlement (e.g. unwind), only the
        // uncommitted suffix `commits[cursor..]` remains in `state.retiring`.
        let _ = self.armed;
    }
}

impl CompositeHistoryCatalog {
    pub(crate) fn begin_durable_retire_changed_history(&self) -> PendingDurableHistoryRetirement {
        let mut state = lock_state(&self.state);
        let mut commits = Vec::new();
        loop {
            let next = lock_index(&state.reachability).pop_changed();
            let Some(commit) = next else {
                break;
            };
            if state.retiring.contains(&commit) {
                continue;
            }
            if EligibleInterior::assess(&state, &commit).is_some() {
                state.retiring.insert(commit.clone());
                commits.push(commit);
            }
        }
        let armed = !commits.is_empty();
        PendingDurableHistoryRetirement {
            state: Arc::clone(&self.state),
            commits,
            cursor: 0,
            mode: DurableRetirementMode::Interior,
            held: None,
            armed,
        }
    }

    pub(crate) fn begin_durable_retire_unprotected_history(
        &self,
        keep: &CompositeCommitIdentity,
    ) -> Result<PendingDurableHistoryRetirement, CompositeHistoryCatalogDenial> {
        let mut state = lock_state(&self.state);
        validate_owner(&state, keep.owner_identity())?;
        if state.retiring.contains(keep) {
            return Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(
                keep.clone(),
            ));
        }
        let ancestry = installed_ancestry(&state, keep)?;
        let mut commits = Vec::new();
        for commit in &ancestry {
            if state.retiring.contains(commit) {
                continue;
            }
            if EligibleInterior::assess(&state, commit).is_some() {
                commits.push(commit.clone());
            }
        }
        commits.reverse();
        for commit in &commits {
            state.retiring.insert(commit.clone());
        }
        let armed = !commits.is_empty();
        Ok(PendingDurableHistoryRetirement {
            state: Arc::clone(&self.state),
            commits,
            cursor: 0,
            mode: DurableRetirementMode::Interior,
            held: None,
            armed,
        })
    }

    pub(crate) fn begin_durable_release_retired_branch_history(
        &self,
        from: &CompositeCommitIdentity,
    ) -> Result<PendingDurableHistoryRetirement, CompositeHistoryCatalogDenial> {
        let mut state = lock_state(&self.state);
        validate_owner(&state, from.owner_identity())?;
        let mut commits = Vec::new();
        let mut held = None;
        let mut simulated_removed = HashSet::new();
        let mut next = Some(from.clone());
        while let Some(commit) = next.take() {
            if state.retiring.contains(&commit) {
                break;
            }
            let Some(entry) = installed(&state, &commit) else {
                break;
            };
            match simulated_retired_branch_need(&state, &commit, &simulated_removed) {
                RetiredBranchNeed::Owned => break,
                RetiredBranchNeed::Held => {
                    held = Some(commit);
                    break;
                }
                RetiredBranchNeed::Unneeded => {}
            }
            next = effective_parent(&state, entry.commit()).map(|(parent, _)| parent);
            simulated_removed.insert(commit.clone());
            commits.push(commit);
        }
        for commit in &commits {
            state.retiring.insert(commit.clone());
        }
        let armed = !commits.is_empty();
        Ok(PendingDurableHistoryRetirement {
            state: Arc::clone(&self.state),
            commits,
            cursor: 0,
            mode: DurableRetirementMode::RetiredBranch,
            held,
            armed,
        })
    }
}

fn simulated_retired_branch_need(
    state: &CompositeHistoryCatalogState,
    commit: &CompositeCommitIdentity,
    simulated_removed: &HashSet<CompositeCommitIdentity>,
) -> RetiredBranchNeed {
    let reachability = lock_index(&state.reachability)
        .lookup(commit)
        .expect("an installed commit has a reachability row");
    if reachability.held_by_product_head() {
        return RetiredBranchNeed::Owned;
    }
    let sole_child_removed = reachability.descendant_dependencies() == 1
        && !reachability.has_forming_child()
        && reachability
            .sole_installed_child()
            .is_some_and(|child| simulated_removed.contains(child));
    if reachability.descendant_dependencies() > 0 && !sole_child_removed {
        return if reachability.has_forming_child() {
            RetiredBranchNeed::Held
        } else {
            RetiredBranchNeed::Owned
        };
    }
    let awaits_delivery = installed(state, commit)
        .and_then(|entry| entry.publication.as_ref())
        .is_some_and(|publication| publication.awaits_delivery());
    if awaits_delivery || reachability.direct_protections() > 0 {
        RetiredBranchNeed::Held
    } else {
        RetiredBranchNeed::Unneeded
    }
}
