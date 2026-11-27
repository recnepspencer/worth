//! Runtime maintenance of each branch's derived persistent-name index.
//!
//! Derived from canonical lineage events installed in publication order; a
//! refused batch marks that branch unavailable until canonical rebuild.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::lineage::data::events::LineageEventRecord;
use crate::lineage::index::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameIndexDenial,
};
use crate::lineage::tiered::NamingReadScope;

use super::group_states::PublishedGroupLineage;
use super::naming_rebuild::named_holders;
use super::recovery_token::FullLineageRecoveryToken;
use super::state::PublishedLineageEvent;
use super::{LineageState, LineageSubsystem};

/// One branch's derived name index, or the refusal that stopped it.
#[derive(Debug)]
pub(super) enum BranchNameIndex {
    Current(PersistentNameBranchIndex),
    Unavailable(PersistentNameIndexDenial),
}

/// Every branch's derived name index.
///
/// Deliberately not `Clone`: [`Self::fork_all`] shares tables copy-on-write.
#[derive(Debug, Default)]
pub(crate) struct BranchNameIndexes {
    pub(super) by_branch: BTreeMap<BranchId, BranchNameIndex>,
}

impl BranchNameIndexes {
    /// The branch's current index, its recorded refusal, or an empty index
    /// for a branch that has published no named lineage yet.
    pub(super) fn read(
        &self,
        branch_id: &BranchId,
    ) -> Result<Option<&PersistentNameBranchIndex>, &PersistentNameIndexDenial> {
        match self.by_branch.get(branch_id) {
            None => Ok(None),
            Some(BranchNameIndex::Current(index)) => Ok(Some(index)),
            Some(BranchNameIndex::Unavailable(denial)) => Err(denial),
        }
    }

    pub(super) fn clear(&mut self) {
        self.by_branch.clear();
    }

    /// Every index for a forked runtime, preserving each branch's own cold base.
    ///
    /// Complexity: O(branches).
    pub(super) fn fork_all(&self) -> Self {
        let by_branch = self
            .by_branch
            .iter()
            .map(|(branch, index)| {
                let index = match index {
                    BranchNameIndex::Current(index) => BranchNameIndex::Current(
                        index.fork_to(branch.clone(), index.base().cloned()),
                    ),
                    BranchNameIndex::Unavailable(denial) => {
                        BranchNameIndex::Unavailable(denial.clone())
                    }
                };
                (branch.clone(), index)
            })
            .collect();
        Self { by_branch }
    }

    /// Ingests one commit's events for `branch_id`, creating a pure-resident
    /// index on its first named lineage.
    ///
    /// Complexity: O(batch events), independent of retained names or commits.
    fn ingest(&mut self, branch_id: &BranchId, batch: &PersistentNameCommittedBatch) {
        let slot = self.by_branch.entry(branch_id.clone()).or_insert_with(|| {
            BranchNameIndex::Current(PersistentNameBranchIndex::over(
                branch_id.clone(),
                None,
                None,
                None,
                None,
            ))
        });
        if let BranchNameIndex::Current(index) = slot
            && let Err(denial) = index.apply(batch)
        {
            *slot = BranchNameIndex::Unavailable(denial);
        }
    }
}

impl LineageSubsystem {
    /// Runs `read` over `branch_id`'s derived name index under the lineage
    /// read lock; `None` is a branch with no named lineage yet. Rows `read`
    /// faults from cold pages live in the boundary's scope; a fault that
    /// failed refuses the whole read typed.
    ///
    /// The lock is held only for `read`, which must not reenter lineage
    /// installation.
    pub fn read_name_index<T>(
        &self,
        branch_id: &BranchId,
        read: impl FnOnce(Option<&PersistentNameBranchIndex>, &NamingReadScope) -> T,
    ) -> Result<T, PersistentNameIndexDenial> {
        let state = self.state.read();
        let scope = NamingReadScope::new();
        let answer = state
            .name_indexes
            .read(branch_id)
            .map(|index| read(index, &scope))
            .map_err(Clone::clone)?;
        scope.settle_index()?;
        Ok(answer)
    }
}

impl LineageState {
    /// Ingests the events installed at positions `from..` (one commit) into
    /// their branch indexes.
    pub(super) fn ingest_installed_name_events(&mut self, from: usize, from_group: usize) {
        let mut name_indexes = std::mem::take(&mut self.name_indexes);
        let groups: Vec<_> = self.published_group_lineage[from_group..].iter().collect();
        let batches = self.name_batches(&self.published_events[from..], &groups);
        for (branch_id, batch) in &batches {
            self.bump_branch_generation(branch_id);
            name_indexes.ingest(branch_id, batch);
        }
        for fork in self.forks_inheriting(&batches) {
            self.bump_branch_generation(&fork);
            let rebuilt = match self.full_history.as_ref() {
                Some(full) => self.rebuild_branch_name_index(full, &fork, CommitId(u64::MAX)),
                None => {
                    BranchNameIndex::Unavailable(PersistentNameIndexDenial::CheckpointedUnavailable)
                }
            };
            name_indexes.by_branch.insert(fork, rebuilt);
        }
        self.name_indexes = name_indexes;
    }

    /// Forked branches whose inherited history reaches one of `batches`,
    /// queried via cutoff-indexed `fork_children` (`O(log C + inheriting)`).
    fn forks_inheriting(
        &self,
        batches: &[(BranchId, PersistentNameCommittedBatch)],
    ) -> Vec<BranchId> {
        if self.fork_children.is_empty() || batches.is_empty() {
            return Vec::new();
        }
        let mut affected = BTreeSet::new();
        let mut stack: Vec<(&BranchId, CommitId, CommitId)> = Vec::new();
        for (branch, batch) in batches {
            if self.fork_children.contains_key(branch) {
                stack.push((branch, batch.commit(), CommitId(u64::MAX)));
            }
        }
        while let Some((parent, batch_commit, inherited_bound)) = stack.pop() {
            let Some(by_cutoff) = self.fork_children.get(parent) else {
                continue;
            };
            for (&fork_commit, children) in by_cutoff.range(batch_commit..) {
                let bound = inherited_bound.min(fork_commit);
                if bound < batch_commit {
                    continue;
                }
                for child in children {
                    if affected.insert(child.clone()) {
                        stack.push((child, batch_commit, bound));
                    }
                }
            }
        }
        affected.into_iter().collect()
    }

    /// Ingests readmitted events that arrived in canonical order.
    pub(super) fn ingest_readmitted_name_events(&mut self, positions: &[usize]) {
        let events: Vec<_> = positions
            .iter()
            .map(|position| self.published_events[*position].clone())
            .collect();
        let mut name_indexes = std::mem::take(&mut self.name_indexes);
        let mut groups: Vec<_> = events
            .iter()
            .filter_map(|p| self.group_lineage_at(p.event.branch_id(), p.publication_commit_id))
            .collect();
        groups.dedup_by_key(|record| record.commit);
        for (branch_id, batch) in self.name_batches(&events, &groups) {
            name_indexes.ingest(&branch_id, &batch);
        }
        self.name_indexes = name_indexes;
    }

    /// Starts `target`'s index from `source`'s at `fork_commit`, enforcing the
    /// retention floor unconditionally before any fast path and preserving
    /// `source`'s own actual cold base when shared unchanged.
    pub(super) fn fork_name_index(
        &mut self,
        target: &BranchId,
        source: &BranchId,
        fork_commit: CommitId,
    ) {
        let inherited = match (
            self.name_indexes.by_branch.get(source),
            self.full_history.as_ref(),
        ) {
            (Some(BranchNameIndex::Current(index)), _)
                if index
                    .retention_floor()
                    .is_some_and(|floor| fork_commit < floor) =>
            {
                BranchNameIndex::Unavailable(PersistentNameIndexDenial::BasisPruned {
                    basis: fork_commit,
                    retention_floor: index.retention_floor().unwrap(),
                })
            }
            (Some(BranchNameIndex::Current(index)), _)
                if index.head().is_none_or(|head| head <= fork_commit) =>
            {
                let link = index.base().map(|base| base.forked(target.clone()));
                BranchNameIndex::Current(index.fork_to(target.clone(), link))
            }
            (Some(BranchNameIndex::Unavailable(denial)), _) => {
                BranchNameIndex::Unavailable(denial.clone())
            }
            (None, None) => BranchNameIndex::Current(PersistentNameBranchIndex::over(
                target.clone(),
                None,
                None,
                None,
                None,
            )),
            (_, Some(full)) => self.rebuild_branch_name_index(full, target, CommitId(u64::MAX)),
            (Some(BranchNameIndex::Current(index)), None) => {
                match index.truncate_at(target.clone(), fork_commit) {
                    Ok(truncated) => BranchNameIndex::Current(truncated),
                    Err(denial) => BranchNameIndex::Unavailable(denial),
                }
            }
        };
        self.name_indexes
            .by_branch
            .insert(target.clone(), inherited);
    }

    /// Rebuilds every branch index from the installed canonical events.
    pub(crate) fn rebuild_name_indexes(&mut self, full: &FullLineageRecoveryToken) {
        full.assert_owner(self.state_id);
        let mut branches: Vec<BranchId> = self.branch_forks.keys().cloned().collect();
        branches.extend(
            self.published_events
                .iter()
                .map(|p| p.event.branch_id().clone()),
        );
        branches.sort();
        branches.dedup();
        let rebuilt = branches
            .into_iter()
            .map(|branch| {
                let index = self.rebuild_branch_name_index(full, &branch, CommitId(u64::MAX));
                (branch, index)
            })
            .collect();
        self.name_indexes = BranchNameIndexes { by_branch: rebuilt };
    }

    /// `branch`'s index through `cutoff` along its fork ancestry.
    fn rebuild_branch_name_index(
        &self,
        _full: &FullLineageRecoveryToken,
        branch: &BranchId,
        cutoff: CommitId,
    ) -> BranchNameIndex {
        let ancestry = self.ancestry(branch, cutoff);
        let mut index: Option<PersistentNameBranchIndex> = None;
        for (ancestor, bound) in ancestry.iter().rev() {
            let mut current = match index.take() {
                Some(inherited) => inherited.rebranched(ancestor.clone()),
                None => PersistentNameBranchIndex::new(ancestor.clone(), None),
            };
            let own: Vec<_> = self
                .published_events
                .iter()
                .filter(|p| p.event.branch_id() == ancestor && p.publication_commit_id <= *bound)
                .cloned()
                .collect();
            let groups = self.own_group_lineage(ancestor, *bound);
            for (_, batch) in self.name_batches(&own, &groups) {
                if let Err(denial) = current.apply(&batch) {
                    return BranchNameIndex::Unavailable(denial);
                }
            }
            index = Some(current);
        }
        BranchNameIndex::Current(index.expect("the ancestry always holds the branch itself"))
    }

    /// `branch` and its fork ancestors, nearest first, with inherited cutoffs.
    pub(super) fn ancestry(
        &self,
        branch: &BranchId,
        cutoff: CommitId,
    ) -> Vec<(BranchId, CommitId)> {
        let mut ancestry = vec![(branch.clone(), cutoff)];
        let mut visited = BTreeSet::from([branch.clone()]);
        while let Some((parent, fork_commit)) = ancestry
            .last()
            .and_then(|(child, _)| self.branch_forks.get(child))
        {
            if !visited.insert(parent.clone()) {
                break;
            }
            let inherited = ancestry.last().map_or(cutoff, |(_, bound)| *bound);
            ancestry.push((parent.clone(), inherited.min(*fork_commit)));
        }
        ancestry
    }

    /// Groups consecutive events by branch and commit into committed batches.
    fn name_batches(
        &self,
        events: &[PublishedLineageEvent],
        groups: &[&PublishedGroupLineage],
    ) -> Vec<(BranchId, PersistentNameCommittedBatch)> {
        let mut batches: Vec<(BranchId, CommitId, Vec<LineageEventRecord>)> = Vec::new();
        for published in events {
            let branch = published.event.branch_id();
            let ev = published.event.as_ref().clone();
            match batches.last_mut() {
                Some((lb, c, run)) if lb == branch && *c == published.publication_commit_id => {
                    run.push(ev);
                }
                _ => batches.push((branch.clone(), published.publication_commit_id, vec![ev])),
            }
        }
        for r in groups {
            if !batches
                .iter()
                .any(|(b, c, _)| *b == r.branch && *c == r.commit)
            {
                batches.push((r.branch.clone(), r.commit, Vec::new()));
            }
        }
        batches.sort_by_key(|(_, commit, _)| *commit);
        batches
            .into_iter()
            .map(|(branch, commit, run)| {
                let (e, r) = named_holders(&run, &self.nodes, &self.relation_lineage_holders);
                let mut batch =
                    PersistentNameCommittedBatch::new(commit).with_lineage_events(run, e, r);
                if let Some(rec) = groups
                    .iter()
                    .find(|g| g.branch == branch && g.commit == commit)
                {
                    batch = batch.with_published_group_transitions(rec.transitions.clone());
                }
                (branch, batch)
            })
            .collect()
    }
}
