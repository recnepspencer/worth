//! Published group lineage and each branch's derived group state.
//!
//! Group lineage is authoritative only as published: one record per commit
//! that carried a canonical group lineage frame. Every branch's group state is
//! derived from those records along its fork ancestry, so a fork, a checkpoint
//! restore, a cold open and a replay all rebuild exactly the state live
//! publication maintained. A record that cannot apply leaves its branch's
//! state unavailable: group finalization and group reads on that branch then
//! refuse typed until canonical history rebuilds it.
//!
//! A cold-backed runtime holds only the records published since its reopen;
//! the earlier log lives in the selected checkpoint's paged group log.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::groups::{BranchGroupState, CommittedGroupTransition, GroupStateDenial};

use super::recovery_token::FullLineageRecoveryToken;
use super::LineageState;

/// One commit's published group transitions.
#[derive(Debug, Clone)]
pub(super) struct PublishedGroupLineage {
    pub(super) branch: BranchId,
    pub(super) commit: CommitId,
    pub(super) transitions: Arc<[CommittedGroupTransition]>,
}

/// Every branch's derived group state, or the refusal that stopped it.
///
/// Deliberately not `Clone`: [`Self::fork_all`] shares tables copy-on-write.
#[derive(Debug, Default)]
pub(crate) struct BranchGroupStates {
    pub(super) by_branch: BTreeMap<BranchId, Result<BranchGroupState, GroupStateDenial>>,
}

impl BranchGroupStates {
    /// Every state for a forked runtime, preserving each branch's own cold base.
    ///
    /// Complexity: O(branches * schemes).
    pub(super) fn fork_all(&self) -> Self {
        let by_branch = self
            .by_branch
            .iter()
            .map(|(branch, state)| {
                let state = match state {
                    Ok(state) => Ok(state.fork(state.base().cloned())),
                    Err(denial) => Err(denial.clone()),
                };
                (branch.clone(), state)
            })
            .collect();
        Self { by_branch }
    }
}

impl LineageState {
    /// Records one commit's group transitions on `branch`, applying them to
    /// the branch's group state. A refused record leaves that branch's state
    /// unavailable; it never refuses the publication that already happened.
    ///
    /// Complexity: O(touched groups and members * log live groups).
    pub(super) fn record_group_lineage(
        &mut self,
        branch: &BranchId,
        commit: CommitId,
        transitions: Vec<CommittedGroupTransition>,
    ) {
        if transitions.is_empty() {
            return;
        }
        self.bump_branch_generation(branch);
        if !self.group_states.by_branch.contains_key(branch) {
            self.group_states
                .by_branch
                .insert(branch.clone(), Ok(BranchGroupState::empty_over(None)));
        }
        if let Some(slot) = self.group_states.by_branch.get_mut(branch)
            && let Ok(state) = slot
            && let Err(denial) = state.apply(commit, &transitions)
        {
            *slot = Err(denial);
        }
        self.published_group_lineage.push(PublishedGroupLineage {
            branch: branch.clone(),
            commit,
            transitions: transitions.into(),
        });
    }

    /// Replaces the published group lineage with canonical recovered records
    /// and rebuilds every branch's group state along the recorded fork
    /// ancestry, refusing the first branch whose records cannot apply. Only a
    /// full-history state can rebuild; a cold-backed one refuses.
    ///
    /// Complexity: O(branches * published group lineage).
    pub fn replace_group_lineage(
        &mut self,
        mut records: Vec<(BranchId, CommitId, Vec<CommittedGroupTransition>)>,
    ) -> Result<(), (BranchId, GroupStateDenial)> {
        if self.full_history.is_none() {
            let branch = records.first().map(|(branch, ..)| branch.clone());
            return Err((
                branch.unwrap_or(BranchId(String::new())),
                GroupStateDenial::CheckpointedUnavailable,
            ));
        }
        records.sort_by_key(|(_, commit, _)| *commit);
        self.published_group_lineage = records
            .into_iter()
            .filter(|(_, _, transitions)| !transitions.is_empty())
            .map(|(branch, commit, transitions)| PublishedGroupLineage {
                branch,
                commit,
                transitions: transitions.into(),
            })
            .collect();
        let full = self
            .full_history
            .take()
            .expect("full_history checked present above");
        self.rebuild_group_states(&full);
        self.full_history = Some(full);
        for (branch, state) in &self.group_states.by_branch {
            if let Err(denial) = state {
                return Err((branch.clone(), denial.clone()));
            }
        }
        Ok(())
    }

    /// `branch`'s group state; `None` before its first group lineage.
    pub(crate) fn branch_group_state(
        &self,
        branch: &BranchId,
    ) -> Result<Option<&BranchGroupState>, &GroupStateDenial> {
        match self.group_states.by_branch.get(branch) {
            None => Ok(None),
            Some(Ok(state)) => Ok(Some(state)),
            Some(Err(denial)) => Err(denial),
        }
    }

    /// The resident published group lineage of `branch` at exactly `commit`.
    ///
    /// Complexity: O(log P) in resident published group lineage records.
    pub(super) fn group_lineage_at(
        &self,
        branch: &BranchId,
        commit: CommitId,
    ) -> Option<&PublishedGroupLineage> {
        let start = self
            .published_group_lineage
            .partition_point(|record| record.commit < commit);
        self.published_group_lineage[start..]
            .iter()
            .take_while(|record| record.commit == commit)
            .find(|record| &record.branch == branch)
    }

    /// Starts `target`'s group state from `source`'s at `fork_commit`,
    /// preserving `source`'s own actual cold base when shared unchanged.
    pub(super) fn fork_group_state(
        &mut self,
        target: &BranchId,
        source: &BranchId,
        fork_commit: CommitId,
    ) {
        let inherited = match (
            self.group_states.by_branch.get(source),
            self.full_history.as_ref(),
        ) {
            (None, _) => return,
            (Some(Ok(state)), _) if state.latest_commit().is_none_or(|at| at <= fork_commit) => {
                let link = state.base().map(|base| base.forked(target.clone()));
                Ok(state.fork(link))
            }
            (Some(Err(denial)), _) => Err(denial.clone()),
            (Some(Ok(_)), Some(full)) => {
                self.rebuild_branch_group_state(full, target, CommitId(u64::MAX))
            }
            (Some(Ok(_)), None) => self.rebuild_historical_group_state(source, fork_commit),
        };
        self.group_states
            .by_branch
            .insert(target.clone(), inherited);
    }

    /// Rebuilds every branch's group state from published group lineage.
    ///
    /// Complexity: O(branches * published group lineage).
    pub(super) fn rebuild_group_states(&mut self, full: &FullLineageRecoveryToken) {
        full.assert_owner(self.state_id);
        let mut branches: Vec<BranchId> = self.branch_forks.keys().cloned().collect();
        branches.extend(
            self.published_group_lineage
                .iter()
                .map(|record| record.branch.clone()),
        );
        branches.sort();
        branches.dedup();
        let by_branch = branches
            .into_iter()
            .map(|branch| {
                let state = self.rebuild_branch_group_state(full, &branch, CommitId(u64::MAX));
                (branch, state)
            })
            .collect();
        self.group_states = BranchGroupStates { by_branch };
    }

    /// `branch`'s group state through `cutoff`, applying each ancestor's own
    /// resident records up to the commit its descendant forked at.
    ///
    /// Complexity: O(published group lineage through `cutoff`); records
    /// after it are never visited.
    pub(super) fn rebuild_branch_group_state(
        &self,
        _full: &FullLineageRecoveryToken,
        branch: &BranchId,
        cutoff: CommitId,
    ) -> Result<BranchGroupState, GroupStateDenial> {
        let bounds: BTreeMap<_, _> = self.ancestry(branch, cutoff).into_iter().collect();
        let mut state = BranchGroupState::default();
        let through = self
            .published_group_lineage
            .partition_point(|record| record.commit <= cutoff);
        for record in &self.published_group_lineage[..through] {
            if bounds
                .get(&record.branch)
                .is_some_and(|bound| record.commit <= *bound)
            {
                state.apply(record.commit, &record.transitions)?;
            }
        }
        Ok(state)
    }

    /// The first group lineage `b`'s history (with fork ancestors) published after `c`,
    /// checking both the selected checkpoint's cold `GROUP_LOG` and the resident tail.
    pub(super) fn group_lineage_after(
        &self,
        b: &BranchId,
        c: CommitId,
    ) -> Result<
        Option<(CommitId, String)>,
        crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    > {
        use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
        use crate::lineage::naming_pages::tables::branch_key_digest;

        let bounds: BTreeMap<_, _> = self.ancestry(b, CommitId(u64::MAX)).into_iter().collect();
        let mut through = None;
        if let Some(source) = &self.cold_naming {
            let store = source.store().ok_or(Cause::SourceUnavailable)?;
            through = store.state().group_log_through.map(CommitId);
            if through.is_some_and(|t| t > c) {
                let digest_bounds: BTreeMap<[u8; 32], CommitId> = bounds
                    .iter()
                    .filter(|&(_, &bound)| bound > c)
                    .map(|(branch, &bound)| (branch_key_digest(branch), bound))
                    .collect();
                if !digest_bounds.is_empty() {
                    let mut found: Option<(CommitId, String)> = None;
                    store.scan_group_log_window(&digest_bounds, Some(c), |at, frame| {
                        let transitions =
                            crate::durability::naming::decode_group_lineage_frame(frame)
                                .map_err(|_| Cause::Damaged)?;
                        if let Some(first) = transitions.first() {
                            if found.as_ref().is_none_or(|(prev, _)| at < *prev) {
                                found = Some((at, first.scheme_id().to_owned()));
                            }
                            return Ok(false);
                        }
                        Ok(true)
                    })?;
                    if found.is_some() {
                        return Ok(found);
                    }
                }
            }
        }
        let record = self.published_group_lineage.iter().find(|record| {
            let bound = bounds.get(&record.branch);
            through.is_none_or(|t| record.commit > t)
                && record.commit > c
                && bound.is_some_and(|bound| record.commit <= *bound)
        });
        Ok(record.and_then(|r| {
            r.transitions
                .first()
                .map(|t| (r.commit, t.scheme_id().to_owned()))
        }))
    }

    /// Published group lineage records of `branch` through `bound`.
    pub(super) fn own_group_lineage(
        &self,
        branch: &BranchId,
        bound: CommitId,
    ) -> Vec<&PublishedGroupLineage> {
        self.published_group_lineage
            .iter()
            .filter(|record| &record.branch == branch && record.commit <= bound)
            .collect()
    }
}
