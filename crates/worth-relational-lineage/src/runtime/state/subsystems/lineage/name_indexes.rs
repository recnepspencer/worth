//! Runtime maintenance of each branch's derived persistent-name index.
//!
//! The index is derived state, never authority: every entry comes from the
//! canonical lineage events the commit pipeline installs, in publication
//! order, and the same events rebuild it on fork, cold open and replay.
//! Live publication installs a commit's events before the branch reference
//! moves, so no observation can select a commit the index has not ingested.
//!
//! A refused batch cannot be repaired by the reader, and it must never refuse
//! the writer that already published canonical truth; that branch's index is
//! recorded unavailable and every name read on it refuses typed until the
//! next rebuild from canonical history.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::events::{
    LineageEventKind, LineageEventRecord, LineageSubject,
};
use crate::contracts::lineage::data::graph::LineageNode;
use crate::lineage::index::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameIndexDenial,
};

use super::state::PublishedLineageEvent;
use super::{LineageState, LineageSubsystem};

/// One branch's derived name index, or the refusal that stopped it.
#[derive(Debug, Clone)]
pub(super) enum BranchNameIndex {
    Current(PersistentNameBranchIndex),
    Unavailable(PersistentNameIndexDenial),
}

/// Every branch's derived name index.
#[derive(Debug, Clone, Default)]
pub(crate) struct BranchNameIndexes {
    by_branch: BTreeMap<BranchId, BranchNameIndex>,
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

    /// Ingests one commit's events for `branch_id`, creating the branch's
    /// index on its first named lineage.
    ///
    /// Complexity: O(batch events), independent of retained names or commits.
    fn ingest(&mut self, branch_id: &BranchId, batch: &PersistentNameCommittedBatch) {
        let slot = self.by_branch.entry(branch_id.clone()).or_insert_with(|| {
            BranchNameIndex::Current(PersistentNameBranchIndex::new(branch_id.clone(), None))
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
    /// read lock; `None` is a branch with no named lineage yet.
    ///
    /// The lock is held only for `read`, which must not reenter lineage
    /// installation.
    pub fn read_name_index<T>(
        &self,
        branch_id: &BranchId,
        read: impl FnOnce(Option<&PersistentNameBranchIndex>) -> T,
    ) -> Result<T, PersistentNameIndexDenial> {
        let state = self.state.read();
        state
            .name_indexes
            .read(branch_id)
            .map(read)
            .map_err(Clone::clone)
    }
}

impl LineageState {
    /// Ingests the events installed at positions `from..` (one commit) into
    /// their branch indexes.
    ///
    /// A branch forked at or after that commit while its settlement was in
    /// flight copied its parent's index without it, so every such fork
    /// descendant is rebuilt from canonical events; forks taken before the
    /// commit never see it and stay as they are.
    ///
    /// Complexity: O(installed events + forks * ancestry depth); a rebuild,
    /// O(installed events), happens only for a fork taken mid-settlement.
    pub(super) fn ingest_installed_name_events(&mut self, from: usize) {
        let mut name_indexes = std::mem::take(&mut self.name_indexes);
        let batches = self.name_batches(&self.published_events[from..]);
        for (branch_id, batch) in &batches {
            name_indexes.ingest(branch_id, batch);
        }
        for fork in self.forks_inheriting(&batches) {
            let rebuilt = self.rebuild_branch_name_index(&fork, CommitId(u64::MAX));
            name_indexes.by_branch.insert(fork, rebuilt);
        }
        self.name_indexes = name_indexes;
    }

    /// Forked branches whose inherited history reaches one of `batches`: a
    /// fork of the batch's branch, directly or through ancestors, whose
    /// inherited cutoff for that branch is at or after the batch's commit.
    fn forks_inheriting(
        &self,
        batches: &[(BranchId, PersistentNameCommittedBatch)],
    ) -> Vec<BranchId> {
        if self.branch_forks.is_empty() || batches.is_empty() {
            return Vec::new();
        }
        self.branch_forks
            .keys()
            .filter(|fork| {
                self.ancestry(fork, CommitId(u64::MAX))
                    .iter()
                    .skip(1)
                    .any(|(ancestor, bound)| {
                        batches
                            .iter()
                            .any(|(branch, batch)| ancestor == branch && *bound >= batch.commit())
                    })
            })
            .cloned()
            .collect()
    }

    /// Ingests readmitted events that arrived in canonical order.
    ///
    /// Complexity: O(readmitted events).
    pub(super) fn ingest_readmitted_name_events(&mut self, positions: &[usize]) {
        let events: Vec<_> = positions
            .iter()
            .map(|position| self.published_events[*position].clone())
            .collect();
        let mut name_indexes = std::mem::take(&mut self.name_indexes);
        for (branch_id, batch) in self.name_batches(&events) {
            name_indexes.ingest(&branch_id, &batch);
        }
        self.name_indexes = name_indexes;
    }

    /// Starts `target`'s index from `source`'s at `fork_commit`, cloning it
    /// when the source has ingested nothing later, else rebuilding it from
    /// canonical events with ancestry cutoffs.
    pub(super) fn fork_name_index(
        &mut self,
        target: &BranchId,
        source: &BranchId,
        fork_commit: CommitId,
    ) {
        let inherited = match self.name_indexes.by_branch.get(source) {
            Some(BranchNameIndex::Current(index))
                if index.head().is_none_or(|head| head <= fork_commit) =>
            {
                BranchNameIndex::Current(index.clone().rebranched(target.clone()))
            }
            None => BranchNameIndex::Current(PersistentNameBranchIndex::new(target.clone(), None)),
            Some(_) => self.rebuild_branch_name_index(target, CommitId(u64::MAX)),
        };
        self.name_indexes
            .by_branch
            .insert(target.clone(), inherited);
    }

    /// Rebuilds every branch index from the installed canonical events.
    ///
    /// Complexity: O(branches * ancestry depth * installed events); a
    /// reconstruction-lane cost paid on cold open and replay only.
    pub(super) fn rebuild_name_indexes(&mut self) {
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
                let index = self.rebuild_branch_name_index(&branch, CommitId(u64::MAX));
                (branch, index)
            })
            .collect();
        self.name_indexes = BranchNameIndexes { by_branch: rebuilt };
    }

    /// `branch`'s index through `cutoff`: its root ancestor's own events,
    /// then each descendant's own events, each bounded by the fork commit
    /// its child inherited. A cycle in fork metadata stops at the first
    /// revisited branch, as the naming basis ancestry does.
    fn rebuild_branch_name_index(&self, branch: &BranchId, cutoff: CommitId) -> BranchNameIndex {
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
            for (_, batch) in self.name_batches(&own) {
                if let Err(denial) = current.apply(&batch) {
                    return BranchNameIndex::Unavailable(denial);
                }
            }
            index = Some(current);
        }
        BranchNameIndex::Current(index.expect("the ancestry always holds the branch itself"))
    }

    /// `branch` and its fork ancestors, nearest first, each with the commit
    /// through which its history reaches `branch`.
    fn ancestry(&self, branch: &BranchId, cutoff: CommitId) -> Vec<(BranchId, CommitId)> {
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

    /// Groups consecutive events by branch and publication commit into
    /// committed batches, binding each named created lineage to its record.
    fn name_batches(
        &self,
        events: &[PublishedLineageEvent],
    ) -> Vec<(BranchId, PersistentNameCommittedBatch)> {
        let mut batches: Vec<(BranchId, CommitId, Vec<LineageEventRecord>)> = Vec::new();
        for published in events {
            let branch = published.event.branch_id();
            match batches.last_mut() {
                Some((last_branch, commit, run))
                    if last_branch == branch && *commit == published.publication_commit_id =>
                {
                    run.push(published.event.as_ref().clone());
                }
                _ => batches.push((
                    branch.clone(),
                    published.publication_commit_id,
                    vec![published.event.as_ref().clone()],
                )),
            }
        }
        batches
            .into_iter()
            .map(|(branch, commit, run)| {
                let (entities, relations) =
                    named_holders(&run, &self.nodes, &self.relation_lineage_holders);
                let batch = PersistentNameCommittedBatch::new(commit)
                    .with_lineage_events(run, entities, relations);
                (branch, batch)
            })
            .collect()
    }
}

/// The records of every lineage a named `Create` or `Restore` binds.
fn named_holders(
    events: &[LineageEventRecord],
    nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
) -> (Vec<(LineageId, EntityId)>, Vec<(LineageId, RelationId)>) {
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let named = events.iter().filter(|event| {
        matches!(
            event.kind(),
            LineageEventKind::Create | LineageEventKind::Restore
        ) && (event.name_birth().is_some() || event.kept_name().is_some())
    });
    for event in named {
        for &lineage in event.targets() {
            match event.subject() {
                LineageSubject::Entity => {
                    if let Some(node) = nodes.get(&lineage) {
                        entities.push((lineage, node.entity_id()));
                    }
                }
                LineageSubject::Relation => {
                    if let Some(relation) = relation_holders.get(&lineage) {
                        relations.push((lineage, *relation));
                    }
                }
            }
        }
    }
    (entities, relations)
}
