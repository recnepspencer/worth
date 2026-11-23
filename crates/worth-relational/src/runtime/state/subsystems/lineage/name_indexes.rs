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

use std::collections::BTreeMap;

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::lineage::data::{LineageEventKind, LineageEventRecord, LineageNode, LineageSubject};
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
    pub(crate) fn read_name_index<T>(
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
    /// Complexity: O(installed events).
    pub(super) fn ingest_installed_name_events(&mut self, from: usize) {
        let mut name_indexes = std::mem::take(&mut self.name_indexes);
        for (branch_id, batch) in self.name_batches(&self.published_events[from..]) {
            name_indexes.ingest(&branch_id, &batch);
        }
        self.name_indexes = name_indexes;
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

    /// `branch`'s index through `cutoff`: its fork parent's index through
    /// the fork commit, re-addressed, followed by its own events.
    fn rebuild_branch_name_index(&self, branch: &BranchId, cutoff: CommitId) -> BranchNameIndex {
        let mut index = match self.branch_forks.get(branch) {
            Some((parent, fork_commit)) if parent != branch => {
                match self.rebuild_branch_name_index(parent, cutoff.min(*fork_commit)) {
                    BranchNameIndex::Current(index) => index.rebranched(branch.clone()),
                    unavailable => return unavailable,
                }
            }
            _ => PersistentNameBranchIndex::new(branch.clone(), None),
        };
        let own: Vec<_> = self
            .published_events
            .iter()
            .filter(|p| p.event.branch_id() == branch && p.publication_commit_id <= cutoff)
            .cloned()
            .collect();
        for (_, batch) in self.name_batches(&own) {
            if let Err(denial) = index.apply(&batch) {
                return BranchNameIndex::Unavailable(denial);
            }
        }
        BranchNameIndex::Current(index)
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
