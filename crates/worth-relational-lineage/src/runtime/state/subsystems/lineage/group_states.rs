//! Published group lineage and each branch's derived group state.
//!
//! Group lineage is authoritative only as published: one record per commit
//! that carried a canonical group lineage frame. Every branch's group state is
//! derived from those records along its fork ancestry, so a fork, a checkpoint
//! restore, a cold open and a replay all rebuild exactly the state live
//! publication maintained. A record that cannot apply leaves its branch's
//! state unavailable: group finalization and group reads on that branch then
//! refuse typed until canonical history rebuilds it.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::groups::{BranchGroupState, CommittedGroupTransition, GroupStateDenial};

use super::LineageState;

/// One commit's published group transitions.
#[derive(Debug, Clone)]
pub(super) struct PublishedGroupLineage {
    pub(super) branch: BranchId,
    pub(super) commit: CommitId,
    pub(super) transitions: Arc<[CommittedGroupTransition]>,
}

/// Every branch's derived group state, or the refusal that stopped it.
#[derive(Debug, Clone, Default)]
pub(crate) struct BranchGroupStates {
    by_branch: BTreeMap<BranchId, Result<BranchGroupState, GroupStateDenial>>,
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
        let slot = self
            .group_states
            .by_branch
            .entry(branch.clone())
            .or_insert_with(|| Ok(BranchGroupState::default()));
        if let Ok(state) = slot
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
    /// ancestry, refusing the first branch whose records cannot apply.
    ///
    /// Complexity: O(branches * published group lineage).
    pub fn replace_group_lineage(
        &mut self,
        mut records: Vec<(BranchId, CommitId, Vec<CommittedGroupTransition>)>,
    ) -> Result<(), (BranchId, GroupStateDenial)> {
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
        self.rebuild_group_states();
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

    /// The published group lineage of `branch` at exactly `commit`.
    ///
    /// Complexity: O(log P) in published group lineage records.
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

    /// Starts `target`'s group state from `source`'s at `fork_commit`.
    pub(super) fn fork_group_state(
        &mut self,
        target: &BranchId,
        source: &BranchId,
        fork_commit: CommitId,
    ) {
        let inherited = match self.group_states.by_branch.get(source) {
            None => return,
            Some(Ok(state)) if state.latest_commit().is_none_or(|at| at <= fork_commit) => {
                Ok(state.clone())
            }
            Some(_) => self.rebuild_branch_group_state(target, CommitId(u64::MAX)),
        };
        self.group_states
            .by_branch
            .insert(target.clone(), inherited);
    }

    /// Rebuilds every branch's group state from published group lineage.
    ///
    /// Complexity: O(branches * published group lineage).
    pub(super) fn rebuild_group_states(&mut self) {
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
                let state = self.rebuild_branch_group_state(&branch, CommitId(u64::MAX));
                (branch, state)
            })
            .collect();
        self.group_states = BranchGroupStates { by_branch };
    }

    /// `branch`'s group state through `cutoff`, applying each ancestor's own
    /// records up to the commit its descendant forked at.
    pub(super) fn rebuild_branch_group_state(
        &self,
        branch: &BranchId,
        cutoff: CommitId,
    ) -> Result<BranchGroupState, GroupStateDenial> {
        let bounds: BTreeMap<_, _> = self.ancestry(branch, cutoff).into_iter().collect();
        let mut state = BranchGroupState::default();
        for record in &self.published_group_lineage {
            if bounds
                .get(&record.branch)
                .is_some_and(|bound| record.commit <= *bound)
            {
                state.apply(record.commit, &record.transitions)?;
            }
        }
        Ok(state)
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
