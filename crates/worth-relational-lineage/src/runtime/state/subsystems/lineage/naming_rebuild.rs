//! Fork seeding and full-history rebuilds of the per-branch naming basis.
//!
//! A fork shares its source's tables copy-on-write. Only a fork taken while
//! a later settlement was in flight needs a rebuild from canonical events,
//! and a rebuild needs the [`FullLineageRecoveryToken`] only a state holding
//! its complete event history carries; a cold-backed state records the fork
//! unavailable instead of deriving it from a partial history.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::events::{
    LineageEventKind, LineageEventRecord, LineageSubject,
};
use crate::contracts::lineage::data::graph::LineageNode;
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};

use super::naming_basis::{
    AuthoritativeBranchNamingState, AuthoritativeLineageBasis, NamingBasisUnavailable,
};
use super::recovery_token::FullLineageRecoveryToken;
use super::state::PublishedLineageEvent;
use super::LineageState;

pub(super) type BranchNaming = Result<AuthoritativeBranchNamingState, NamingBasisUnavailable>;

/// `branch`'s naming basis replayed through `cutoff` from every published event
/// along its fork ancestry.
///
/// Complexity: O(published events); full-history recovery only.
pub(super) fn rebuild_branch_naming_state_for_branch(
    _full: &FullLineageRecoveryToken,
    branch_id: &BranchId,
    cutoff: CommitId,
    published_events: &[PublishedLineageEvent],
    entity_nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
    branch_forks: &BTreeMap<BranchId, (BranchId, CommitId)>,
) -> BranchNaming {
    let cutoffs = ancestor_commit_cutoffs(branch_id, cutoff, branch_forks);
    let mut state = AuthoritativeBranchNamingState::over(None, None, None);
    for published in published_events {
        let Some(&max_commit) = cutoffs.get(published.event.branch_id()) else {
            continue;
        };
        if published.publication_commit_id <= max_commit {
            state
                .apply_event(
                    &published.event,
                    published.publication_commit_id,
                    entity_nodes,
                    relation_holders,
                )
                .map_err(NamingBasisUnavailable::Storage)?;
        }
    }
    Ok(state)
}

/// Every branch's naming basis replayed from canonical events.
///
/// Complexity: O(branches * published events); full-history recovery only.
pub(super) fn rebuild_all_branch_naming_states(
    full: &FullLineageRecoveryToken,
    published_events: &[PublishedLineageEvent],
    entity_nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
    branch_forks: &BTreeMap<BranchId, (BranchId, CommitId)>,
) -> BTreeMap<BranchId, BranchNaming> {
    let mut branches = BTreeSet::new();
    for published in published_events {
        branches.insert(published.event.branch_id().clone());
    }
    branches.extend(branch_forks.keys().cloned());
    branches
        .into_iter()
        .map(|branch_id| {
            let state = rebuild_branch_naming_state_for_branch(
                full,
                &branch_id,
                CommitId(u64::MAX),
                published_events,
                entity_nodes,
                relation_holders,
                branch_forks,
            );
            (branch_id, state)
        })
        .collect()
}

fn ancestor_commit_cutoffs(
    branch_id: &BranchId,
    cutoff: CommitId,
    branch_forks: &BTreeMap<BranchId, (BranchId, CommitId)>,
) -> BTreeMap<BranchId, CommitId> {
    let mut cutoffs = BTreeMap::new();
    let mut current = branch_id.clone();
    let mut current_cutoff = cutoff;
    while !cutoffs.contains_key(&current) {
        cutoffs.insert(current.clone(), current_cutoff);
        let Some((parent, fork_commit)) = branch_forks.get(&current) else {
            break;
        };
        current_cutoff = current_cutoff.min(*fork_commit);
        current = parent.clone();
    }
    cutoffs
}

impl LineageState {
    pub fn record_relation_lineage(&mut self, relation_id: RelationId, lineage_id: LineageId) {
        self.relation_lineages.insert(relation_id, lineage_id);
        self.relation_lineage_holders
            .insert(lineage_id, relation_id);
    }

    pub(crate) fn relation_lineage(&self, relation_id: RelationId) -> Option<LineageId> {
        if let Some(&lineage) = self.relation_lineages.get(&relation_id) {
            return Some(lineage);
        }
        let holder =
            crate::contracts::lineage::data::naming::PersistentNameHolder::Relation(relation_id);
        let scope = crate::lineage::tiered::NamingReadScope::new();
        for slot in self.branch_naming_states.values() {
            if let Ok(state) = slot
                && let Some(lineage) = state.live_lineage_in(&holder, &scope)
            {
                return Some(lineage);
            }
        }
        for slot in self.name_indexes.by_branch.values() {
            if let super::name_indexes::BranchNameIndex::Current(index) = slot
                && let Some(lineage) = index.latest_holder_lineage_in(
                    crate::lineage::index::LineageHolder::Relation(relation_id),
                    &scope,
                )
            {
                return Some(lineage);
            }
        }
        None
    }

    pub(crate) fn relation_lineages_snapshot(&self) -> Vec<(RelationId, LineageId)> {
        self.relation_lineages
            .iter()
            .map(|(&r, &l)| (r, l))
            .collect()
    }

    pub fn record_fork_ancestry_metadata(
        &mut self,
        target: BranchId,
        source: BranchId,
        fork_commit_id: CommitId,
    ) {
        if let Some((old_source, old_cutoff)) = self
            .branch_forks
            .insert(target.clone(), (source.clone(), fork_commit_id))
            && let Some(by_cutoff) = self.fork_children.get_mut(&old_source)
        {
            if let Some(children) = by_cutoff.get_mut(&old_cutoff) {
                children.remove(&target);
                if children.is_empty() {
                    by_cutoff.remove(&old_cutoff);
                }
            }
            if by_cutoff.is_empty() {
                self.fork_children.remove(&old_source);
            }
        }
        self.fork_children
            .entry(source)
            .or_default()
            .entry(fork_commit_id)
            .or_default()
            .insert(target);
    }

    pub(super) fn branch_naming_state(&self, branch_id: &BranchId) -> Option<&BranchNaming> {
        self.branch_naming_states.get(branch_id)
    }

    pub(crate) fn holder_and_name_for_lineage(
        &self,
        branch_id: &BranchId,
        lineage_id: LineageId,
    ) -> Result<Option<(PersistentNameHolder, PersistentNameIdentity)>, NamingBasisUnavailable>
    {
        match self.branch_naming_states.get(branch_id) {
            None => Ok(None),
            Some(Err(denial)) => Err(*denial),
            Some(Ok(state)) => state
                .holder_and_name_for_lineage(lineage_id)
                .map_err(NamingBasisUnavailable::Storage),
        }
    }

    /// Complexity: O(N) full scan; audits and courts only.
    pub fn authoritative_branch_naming_basis(
        &self,
        branch_id: &BranchId,
    ) -> AuthoritativeLineageBasis {
        match self.branch_naming_states.get(branch_id) {
            None => AuthoritativeLineageBasis::default(),
            Some(Ok(state)) => state
                .snapshot()
                .unwrap_or_else(|cause| panic!("audit of a damaged naming page: {cause:?}")),
            Some(Err(denial)) => panic!("audit of an unavailable naming basis: {denial:?}"),
        }
    }

    /// Complexity: O(live names) full scan; returns typed error on damaged cold
    /// pages or an unavailable naming basis instead of panicking.
    pub(crate) fn branch_live_persistent_names(
        &self,
        branch_id: &BranchId,
    ) -> Result<BTreeMap<PersistentNameIdentity, PersistentNameHolder>, NamingBasisUnavailable>
    {
        match self.branch_naming_states.get(branch_id) {
            None => Ok(BTreeMap::new()),
            Some(Ok(state)) => state
                .live_holders()
                .map_err(NamingBasisUnavailable::Storage),
            Some(Err(denial)) => Err(*denial),
        }
    }
}

/// The records of every lineage a `Create` or `Restore` binds (both named and
/// unnamed, so historical fork slicing preserves unnamed lineage bindings).
pub(super) fn named_holders(
    events: &[LineageEventRecord],
    nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
) -> (Vec<(LineageId, EntityId)>, Vec<(LineageId, RelationId)>) {
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let bound = events.iter().filter(|event| {
        matches!(
            event.kind(),
            LineageEventKind::Create | LineageEventKind::Restore
        )
    });
    for event in bound {
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

/// Exact `(earliest, latest)` lineage event commit bounds visible on
/// `branch_id` through `cutoff` when full event history is resident.
///
/// Queries only ancestor branches in `event_commits_by_branch`, ignoring all
/// unrelated branch history (`O(ancestors * log H_branch)`).
pub(super) fn branch_lineage_event_commit_bounds(
    _full: &FullLineageRecoveryToken,
    branch_id: &BranchId,
    cutoff: CommitId,
    event_commits_by_branch: &BTreeMap<BranchId, BTreeSet<CommitId>>,
    branch_forks: &BTreeMap<BranchId, (BranchId, CommitId)>,
) -> (Option<CommitId>, Option<CommitId>) {
    let cutoffs = ancestor_commit_cutoffs(branch_id, cutoff, branch_forks);
    let (mut earliest, mut latest): (Option<CommitId>, Option<CommitId>) = (None, None);
    for (ancestor, max_commit) in cutoffs {
        let Some(commits) = event_commits_by_branch.get(&ancestor) else {
            continue;
        };
        if let Some(first) = commits.first().copied().filter(|&c| c <= max_commit) {
            earliest = Some(earliest.map_or(first, |prev| prev.min(first)));
        }
        if let Some(last) = commits.range(..=max_commit).next_back().copied() {
            latest = Some(latest.map_or(last, |prev| prev.max(last)));
        }
    }
    (earliest, latest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::identity::data::primitives::{LineageId, PartitionId};
    use crate::contracts::identity::data::record_ids::EntityId;
    use crate::contracts::lineage::data::events::LineageEventKind;
    use crate::contracts::lineage::data::graph::assemble_lineage_node;
    use crate::lineage::naming_pages::tests::unnamed_retirement_and_group_log_carry::unnamed_event;

    #[test]
    fn event_commit_bounds_respect_multi_hop_cutoffs_ignoring_unrelated_history() {
        let mut state = LineageState::empty();
        let main = BranchId("main".into());
        let mid = BranchId("mid".into());
        let leaf = BranchId("leaf".into());
        let mut next_event_id = 1u64;
        let mut next_lineage_id = 1u64;
        let mut push_ev = |state: &mut LineageState, branch: &BranchId, commit: u64| {
            let lid = LineageId(next_lineage_id);
            next_lineage_id += 1;
            let eid = EntityId::new(PartitionId::main(), next_lineage_id, 1);
            state.record_node(assemble_lineage_node(lid, eid));
            let ev = unnamed_event(
                next_event_id,
                branch,
                CommitId(commit),
                LineageEventKind::Create,
                vec![],
                vec![lid],
            );
            next_event_id += 1;
            state
                .install_recovered_event_batch(&[ev], CommitId(commit))
                .unwrap();
        };

        // Flood 5,000 events across 50 unrelated branches spanning commits 1..=10_000.
        for b in 0..50u64 {
            let unrelated = BranchId(format!("unrelated-{b:02}"));
            for k in 0..100u64 {
                push_ev(&mut state, &unrelated, 1 + k * 100 + b);
            }
        }

        // Multi-hop ancestry: main (commits 5, 10, 50) -> mid@20 (25, 35, 60) -> leaf@40 (42, 48, 80).
        for c in [5, 10, 50] {
            push_ev(&mut state, &main, c);
        }
        state.record_fork_ancestry_metadata(mid.clone(), main.clone(), CommitId(20));
        for c in [25, 35, 60] {
            push_ev(&mut state, &mid, c);
        }
        state.record_fork_ancestry_metadata(leaf.clone(), mid.clone(), CommitId(40));
        for c in [42, 48, 80] {
            push_ev(&mut state, &leaf, c);
        }

        let full = state.full_history.as_ref().expect("full history resident");
        assert_eq!(
            branch_lineage_event_commit_bounds(
                full,
                &leaf,
                CommitId(45),
                &state.event_commits_by_branch,
                &state.branch_forks,
            ),
            (Some(CommitId(5)), Some(CommitId(42)))
        );
        assert_eq!(
            branch_lineage_event_commit_bounds(
                full,
                &leaf,
                CommitId(30),
                &state.event_commits_by_branch,
                &state.branch_forks,
            ),
            (Some(CommitId(5)), Some(CommitId(25)))
        );
        assert_eq!(
            branch_lineage_event_commit_bounds(
                full,
                &leaf,
                CommitId(4),
                &state.event_commits_by_branch,
                &state.branch_forks,
            ),
            (None, None)
        );
        assert_eq!(
            branch_lineage_event_commit_bounds(
                full,
                &BranchId("never-seen".into()),
                CommitId(10_000),
                &state.event_commits_by_branch,
                &state.branch_forks,
            ),
            (None, None)
        );
    }
}
