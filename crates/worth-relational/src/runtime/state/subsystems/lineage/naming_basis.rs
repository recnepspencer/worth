//! Authoritative branch-local persistent-name basis derived solely from
//! Relational's committed `LineageEventRecord`s, entity `LineageNode`s,
//! persisted relation lineage bindings, and fork ancestry.
//!
//! Commit finalization queries this derived basis through a borrowed
//! [`BranchNamingBasisGuard`] implementing [`LineageBasisView`] with O(delta)
//! point lookups and zero map clones, and reconstruction
//! (`LineageState::rebuild_derived_indexes`) rebuilds it across fork ancestry
//! from canonical events and holder bindings alone.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::RwLockReadGuard;

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{LineageId, RelationId};
use crate::lineage::data::{
    LineageBasisView, LineageEventKind, LineageEventRecord, LineageNode, LineageSubject,
    PersistentNameHolder, PersistentNameIdentity,
};
use crate::runtime::state::subsystems::RuntimeOwnedState;

use super::state::PublishedLineageEvent;
use super::LineageState;

/// Snapshot of live persistent-name holders, live record lineages, and
/// previously retired persistent-name lineages for one branch (test-only
/// basis-equality helper; live commits borrow via `BranchNamingBasisGuard`).
#[cfg(test)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AuthoritativeLineageBasis {
    pub(crate) live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub(crate) live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    pub(crate) retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
}

/// Incrementally maintained, rebuildable per-branch naming basis inside
/// `LineageState`.
#[derive(Debug, Clone, Default)]
pub(super) struct AuthoritativeBranchNamingState {
    lineage_holders: BTreeMap<LineageId, PersistentNameHolder>,
    holder_names: BTreeMap<PersistentNameHolder, PersistentNameIdentity>,
    live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
    latest_event_commit: Option<CommitId>,
}

impl AuthoritativeBranchNamingState {
    #[cfg(test)]
    pub(super) fn snapshot(&self) -> AuthoritativeLineageBasis {
        AuthoritativeLineageBasis {
            live_holders: self.live_holders.clone(),
            live_lineages: self.live_lineages.clone(),
            retired_lineages: self.retired_lineages.clone(),
        }
    }

    pub(super) fn live_holders(&self) -> &BTreeMap<PersistentNameIdentity, PersistentNameHolder> {
        &self.live_holders
    }

    pub(super) fn latest_event_commit(&self) -> Option<CommitId> {
        self.latest_event_commit
    }

    pub(super) fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder> {
        self.live_holders.get(id)
    }

    pub(super) fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId> {
        self.live_lineages.get(holder).copied()
    }

    pub(super) fn retired_lineage(
        &self,
        id: &PersistentNameIdentity,
    ) -> Option<(LineageSubject, LineageId)> {
        self.retired_lineages.get(id).copied()
    }

    pub(super) fn holder_and_name_for_lineage(
        &self,
        lineage_id: LineageId,
    ) -> Option<(PersistentNameHolder, PersistentNameIdentity)> {
        let holder = self.lineage_holders.get(&lineage_id)?.clone();
        let name = *self.holder_names.get(&holder)?;
        Some((holder, name))
    }

    pub(super) fn apply_event(
        &mut self,
        event: &LineageEventRecord,
        publication_commit_id: CommitId,
        entity_nodes: &BTreeMap<LineageId, LineageNode>,
        relation_holders: &BTreeMap<LineageId, RelationId>,
    ) {
        self.latest_event_commit = Some(
            self.latest_event_commit
                .map_or(publication_commit_id, |prev| prev.max(publication_commit_id)),
        );
        match event.kind() {
            LineageEventKind::Create | LineageEventKind::Restore => {
                let Some(&target_lineage) = event.targets().first() else {
                    return;
                };
                let Some(holder) =
                    resolve_holder(event.subject(), target_lineage, entity_nodes, relation_holders)
                else {
                    return;
                };
                self.lineage_holders.insert(target_lineage, holder.clone());
                self.live_lineages.insert(holder.clone(), target_lineage);
                if let Some(birth) = event.name_birth() {
                    let identity = birth.identity();
                    self.holder_names.insert(holder.clone(), identity);
                    self.live_holders.insert(identity, holder);
                    self.retired_lineages.remove(&identity);
                } else if let Some(kept) = event.kept_name() {
                    self.holder_names.insert(holder.clone(), kept);
                    self.live_holders.insert(kept, holder);
                    self.retired_lineages.remove(&kept);
                }
            }
            LineageEventKind::Retire
            | LineageEventKind::Replace
            | LineageEventKind::Split
            | LineageEventKind::Merge => {
                for &source in event.sources() {
                    if event.targets().contains(&source) {
                        continue;
                    }
                    let Some(holder) = self.lineage_holders.get(&source).cloned().or_else(|| {
                        resolve_holder(event.subject(), source, entity_nodes, relation_holders)
                    }) else {
                        continue;
                    };
                    self.live_lineages.remove(&holder);
                    if let Some(&name) = self.holder_names.get(&holder)
                        && self.live_holders.get(&name) == Some(&holder)
                    {
                        self.live_holders.remove(&name);
                        self.retired_lineages
                            .insert(name, (event.subject(), source));
                    }
                }
            }
        }
    }
}

pub(super) fn rebuild_branch_naming_state_for_branch(
    branch_id: &BranchId,
    published_events: &[PublishedLineageEvent],
    entity_nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
    branch_forks: &BTreeMap<BranchId, (BranchId, CommitId)>,
) -> AuthoritativeBranchNamingState {
    let cutoffs = ancestor_commit_cutoffs(branch_id, branch_forks);
    let mut state = AuthoritativeBranchNamingState::default();
    for published in published_events {
        let Some(&max_commit) = cutoffs.get(published.event.branch_id()) else {
            continue;
        };
        if published.publication_commit_id <= max_commit {
            state.apply_event(
                &published.event,
                published.publication_commit_id,
                entity_nodes,
                relation_holders,
            );
        }
    }
    state
}

pub(super) fn rebuild_all_branch_naming_states(
    published_events: &[PublishedLineageEvent],
    entity_nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
    branch_forks: &BTreeMap<BranchId, (BranchId, CommitId)>,
) -> BTreeMap<BranchId, AuthoritativeBranchNamingState> {
    let mut branches = BTreeSet::new();
    for published in published_events {
        branches.insert(published.event.branch_id().clone());
    }
    for target in branch_forks.keys() {
        branches.insert(target.clone());
    }
    let mut states = BTreeMap::new();
    for branch_id in branches {
        let state = rebuild_branch_naming_state_for_branch(
            &branch_id,
            published_events,
            entity_nodes,
            relation_holders,
            branch_forks,
        );
        states.insert(branch_id, state);
    }
    states
}

fn ancestor_commit_cutoffs(
    branch_id: &BranchId,
    branch_forks: &BTreeMap<BranchId, (BranchId, CommitId)>,
) -> BTreeMap<BranchId, CommitId> {
    let mut cutoffs = BTreeMap::new();
    let mut current = branch_id.clone();
    let mut current_cutoff = CommitId(u64::MAX);
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

fn resolve_holder(
    subject: LineageSubject,
    lineage_id: LineageId,
    entity_nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
) -> Option<PersistentNameHolder> {
    match subject {
        LineageSubject::Entity => entity_nodes
            .get(&lineage_id)
            .map(|node| PersistentNameHolder::Entity(node.entity_id())),
        LineageSubject::Relation => relation_holders
            .get(&lineage_id)
            .copied()
            .map(PersistentNameHolder::Relation),
    }
}

/// Zero-clone read-lock guard exposing O(log N) point lookups into one branch's
/// authoritative naming basis for commit finalization.
pub(crate) struct BranchNamingBasisGuard<'a> {
    guard: RwLockReadGuard<'a, LineageState>,
    branch_id: BranchId,
}

impl LineageBasisView for BranchNamingBasisGuard<'_> {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.live_holder(id))
    }

    fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.live_lineage(holder))
    }

    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        self.guard
            .branch_naming_state(&self.branch_id)
            .and_then(|state| state.retired_lineage(id))
    }
}

/// Narrow read/fork-registration binding handed to `RelationalPreparationRuntime`
/// and `RelationalForkPort`.
#[derive(Debug)]
pub(crate) struct LineageNamingBasisBinding {
    state: RuntimeOwnedState<LineageState>,
}

impl Clone for LineageNamingBasisBinding {
    fn clone(&self) -> Self {
        Self {
            state: self.state.share(),
        }
    }
}

impl LineageNamingBasisBinding {
    pub(super) fn new(state: RuntimeOwnedState<LineageState>) -> Self {
        Self { state }
    }

    pub(crate) fn read_guard(&self, branch_id: BranchId) -> BranchNamingBasisGuard<'_> {
        BranchNamingBasisGuard {
            guard: self.state.read(),
            branch_id,
        }
    }

    pub(crate) fn register_fork_ancestry(
        &self,
        target_branch: BranchId,
        source_branch: BranchId,
        fork_commit_id: CommitId,
    ) {
        self.state
            .write()
            .register_fork_ancestry(target_branch, source_branch, fork_commit_id);
    }

    #[cfg(test)]
    pub(crate) fn authoritative_basis(&self, branch_id: &BranchId) -> AuthoritativeLineageBasis {
        self.state.read().authoritative_branch_naming_basis(branch_id)
    }
}

impl LineageState {
    pub(crate) fn record_relation_lineage(&mut self, relation_id: RelationId, lineage_id: LineageId) {
        self.relation_lineages.insert(relation_id, lineage_id);
        self.relation_lineage_holders.insert(lineage_id, relation_id);
    }

    pub(crate) fn relation_lineage(&self, relation_id: RelationId) -> Option<LineageId> {
        self.relation_lineages.get(&relation_id).copied()
    }

    pub(crate) fn relation_lineages_snapshot(&self) -> Vec<(RelationId, LineageId)> {
        self.relation_lineages.iter().map(|(&r, &l)| (r, l)).collect()
    }

    pub(crate) fn record_fork_ancestry_metadata(&mut self, target: BranchId, source: BranchId, fork_commit_id: CommitId) {
        self.branch_forks.insert(target, (source, fork_commit_id));
    }

    pub(crate) fn register_fork_ancestry(&mut self, target: BranchId, source: BranchId, fork_commit_id: CommitId) {
        self.branch_forks.insert(target.clone(), (source.clone(), fork_commit_id));
        let can_clone_head = self.branch_naming_states.get(&source).is_some_and(|s| {
            s.latest_event_commit().is_none_or(|latest| latest <= fork_commit_id)
        });
        let seeded = if can_clone_head {
            self.branch_naming_states.get(&source).cloned().unwrap_or_default()
        } else {
            rebuild_branch_naming_state_for_branch(
                &target, &self.published_events, &self.nodes, &self.relation_lineage_holders, &self.branch_forks,
            )
        };
        self.branch_naming_states.insert(target, seeded);
    }

    pub(super) fn branch_naming_state(&self, branch_id: &BranchId) -> Option<&AuthoritativeBranchNamingState> {
        self.branch_naming_states.get(branch_id)
    }

    pub(crate) fn holder_and_name_for_lineage(
        &self,
        branch_id: &BranchId,
        lineage_id: LineageId,
    ) -> Option<(PersistentNameHolder, PersistentNameIdentity)> {
        self.branch_naming_states
            .get(branch_id)
            .and_then(|s| s.holder_and_name_for_lineage(lineage_id))
    }

    #[cfg(test)]
    pub(crate) fn authoritative_branch_naming_basis(&self, branch_id: &BranchId) -> AuthoritativeLineageBasis {
        self.branch_naming_states.get(branch_id).map(AuthoritativeBranchNamingState::snapshot).unwrap_or_default()
    }

    pub(crate) fn branch_live_persistent_names(
        &self,
        branch_id: &BranchId,
    ) -> BTreeMap<PersistentNameIdentity, PersistentNameHolder> {
        self.branch_naming_states.get(branch_id).map(|s| s.live_holders().clone()).unwrap_or_default()
    }
}
