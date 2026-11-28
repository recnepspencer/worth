//! Installation of a checkpoint's paged naming state without reading a page.
//!
//! Reopen installs, per branch the checkpoint recorded, the resident scalar
//! headers and tables whose rows stay cold behind a [`NamingBaseLink`]: it
//! reads no naming, group, lineage node or lineage event page. The installed
//! state drops the full-history capability, so no full-history rebuild can
//! run on it, and it never ingests readmitted lineage events into naming:
//! the cold base already reflects every checkpointed event.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState;

use crate::contracts::aggregate_identity::transition::group::model::AggregateContinuityRule;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::lineage::groups::{BranchGroupState, GroupStateDenial};
use crate::lineage::index::{PersistentNameBranchIndex, PersistentNameIndexDenial};
use crate::lineage::naming_pages::CheckpointNamingState;
use crate::lineage::tiered::{ColdNamingSource, NamingBaseLink};

use super::name_indexes::BranchNameIndex;
use super::{LineageState, LineageSubsystem};

/// Weakly-referenced hook invoked when a checkpoint is acknowledged so the
/// lineage subsystem consolidates unchanged resident deltas onto the newly
/// acknowledged cold naming base without holding a reference cycle.
#[derive(Debug)]
pub struct LineageNamingAckHook {
    state: RuntimeOwnedState<LineageState>,
}

/// Non-blocking write-lock guard acquired before checkpoint ACK mutates the
/// selected directory so directory swap and naming consolidation serialize
/// atomically without deadlocking read callbacks that inspect cold stats.
pub struct LineageNamingAckGuard<'a>(std::sync::RwLockWriteGuard<'a, LineageState>);

impl LineageNamingAckGuard<'_> {
    pub fn consolidate_acknowledged_naming(
        mut self,
        source: Arc<dyn ColdNamingSource>,
        naming: &CheckpointNamingState,
        captured_generations: &BTreeMap<BranchId, u64>,
    ) {
        consolidate_locked(&mut self.0, source, naming, captured_generations);
    }
}

impl LineageNamingAckHook {
    pub(super) fn new(state: RuntimeOwnedState<LineageState>) -> Self {
        Self { state }
    }

    /// Acquires the lineage write lock without blocking; returns `None` when a
    /// concurrent reader/writer holds it so ACK yields `Busy` instead of
    /// deadlocking against read callbacks that acquire the selection mutex.
    pub fn try_lock_for_ack(&self) -> Option<LineageNamingAckGuard<'_>> {
        self.state.try_write().map(LineageNamingAckGuard)
    }
}

fn consolidate_locked(
    state: &mut LineageState,
    source: Arc<dyn ColdNamingSource>,
    naming: &CheckpointNamingState,
    captured_generations: &BTreeMap<BranchId, u64>,
) {
    state.cold_naming = Some(source.clone());
    let mut cleaned = std::collections::BTreeSet::new();
    for root in &naming.branches {
        let captured = captured_generations.get(&root.branch).copied().unwrap_or(0);
        let current = state
            .branch_generations
            .get(&root.branch)
            .copied()
            .unwrap_or(0);
        if current != captured {
            continue;
        }
        let link = NamingBaseLink::new(source.clone(), root.branch.clone());
        if let Some(BranchNameIndex::Current(index)) =
            state.name_indexes.by_branch.get_mut(&root.branch)
        {
            index.rebind_clean(link.clone());
        }
        if let Some(Ok(groups)) = state.group_states.by_branch.get_mut(&root.branch) {
            groups.rebind_clean(link);
        }
        cleaned.insert(root.branch.clone());
    }
    // Rebind every dirty or post-capture forked branch onto its nearest
    // ancestor present in `naming` so `NamingBaseLink` stays O(1) and
    // never resolves against an ancestor that mutated after the fork.
    let remaining: Vec<BranchId> = state
        .name_indexes.by_branch.keys()
        .chain(state.group_states.by_branch.keys())
        .filter(|b| !cleaned.contains(*b))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for branch in remaining {
        use std::ops::ControlFlow::{Break, Continue};
        let Ok(nearest) = worth_relational_branch::internal_port::history::access::ancestry_walk::walk_acyclic_chain(
            branch.clone(),
            &mut std::collections::BTreeSet::new(),
            || Ok::<_, std::convert::Infallible>(()),
            |cursor| {
                Ok(if naming.root(&cursor).is_some() {
                    Break(Some(cursor))
                } else if let Some((parent, _)) = state.branch_forks.get(&cursor) {
                    Continue(parent.clone())
                } else {
                    Break(None)
                })
            },
        );
        let nearest = nearest.flatten();
        let Some(anc) = nearest else {
            continue;
        };
        if let Some(BranchNameIndex::Current(index)) = state.name_indexes.by_branch.get_mut(&branch)
        {
            index.rebind_source(&source, Some(&anc));
        }
        if let Some(Ok(groups)) = state.group_states.by_branch.get_mut(&branch) {
            groups.rebind_source(&source, Some(&anc));
        }
    }
    if let Some(through) = naming.group_log_through {
        // Revoke full resident history when covered group log records are
        // trimmed onto the cold checkpoint so historical checkout and fork
        // consistently merge cold GROUP_LOG + resident tail.
        state.full_history = None;
        state
            .published_group_lineage
            .retain(|record| record.commit.0 > through);
    }
}

pub(super) fn rule_of(tag: u8) -> Option<AggregateContinuityRule> {
    match tag {
        1 => Some(AggregateContinuityRule::Anchor),
        2 => Some(AggregateContinuityRule::Boundary),
        3 => Some(AggregateContinuityRule::ProducerRole),
        _ => None,
    }
}

impl LineageSubsystem {
    pub fn new() -> Self {
        let state = RuntimeOwnedState::new(LineageState::empty());
        let ack_hook = Arc::new(LineageNamingAckHook::new(state.share()));
        Self {
            discovery_metadata: Default::default(),
            state,
            ack_hook,
            node_residency: Default::default(),
            event_residency: Default::default(),
            event_admissions: Default::default(),
        }
    }

    pub fn fork(&self) -> Self {
        self.fork_with_pinned_cold().0
    }

    /// Installs `naming`'s headers as each recorded branch's naming basis,
    /// name index and group state, with every table cold behind `source`.
    ///
    /// Complexity: O(branches + schemes); reads no page.
    pub fn install_cold_naming(
        &self,
        source: Arc<dyn ColdNamingSource>,
        naming: &CheckpointNamingState,
    ) {
        let mut state = self.state.write();
        state.full_history = None;
        state.naming_rebuilt_event_frontier = Some(u64::MAX);
        state.event_commits_by_branch.clear();
        state.published_group_lineage.clear();
        state.name_indexes.clear();
        state.group_states.by_branch.clear();
        for root in &naming.branches {
            let branch: BranchId = root.branch.clone();
            let link = NamingBaseLink::new(source.clone(), branch.clone());
            if let Some(header) = root.index {
                let index = if header.unavailable {
                    BranchNameIndex::Unavailable(PersistentNameIndexDenial::CheckpointedUnavailable)
                } else {
                    BranchNameIndex::Current(PersistentNameBranchIndex::over(
                        branch.clone(),
                        header.retention_floor.map(CommitId),
                        header.earliest_commit.map(CommitId),
                        header.head.map(CommitId),
                        Some(link.clone()),
                    ))
                };
                state.name_indexes.by_branch.insert(branch.clone(), index);
            }
            if let Some(header) = &root.groups {
                let groups = if header.unavailable {
                    Err(GroupStateDenial::CheckpointedUnavailable)
                } else {
                    let schemes = header
                        .schemes
                        .iter()
                        .filter_map(|scheme| Some((scheme.id.clone(), rule_of(scheme.rule)?)));
                    Ok(BranchGroupState::over(
                        schemes,
                        header.earliest_commit.map(CommitId),
                        header.latest_commit.map(CommitId),
                        Some(link.clone()),
                    ))
                };
                state.group_states.by_branch.insert(branch, groups);
            }
        }
        for edge in &naming.retired_forks {
            state
                .branch_forks
                .entry(edge.branch.clone())
                .or_insert((edge.parent.clone(), CommitId(edge.fork_commit)));
        }
        state.cold_naming = Some(source);
    }

    /// Atomically forks this subsystem together with a custody-pinned handle to
    /// the exact cold naming store backing the forked routes, so a concurrent
    /// checkpoint ACK cannot mix pre-ACK routes with a post-ACK directory.
    pub fn fork_with_pinned_cold(
        &self,
    ) -> (Self, Option<crate::lineage::tiered::PinnedNamingStore>) {
        let admissions = self.event_admissions.read();
        let (detached_state, pinned) = {
            let state = self.state.read();
            let pinned = state.cold_naming.as_ref().and_then(|source| source.store());
            (state.fork_detached(), pinned)
        };
        let state = RuntimeOwnedState::new(detached_state);
        let ack_hook = Arc::new(LineageNamingAckHook::new(state.share()));
        let forked = Self {
            discovery_metadata: Default::default(),
            state,
            ack_hook,
            node_residency: self.node_residency.detached(),
            event_residency: self.event_residency.detached(),
            event_admissions: RuntimeOwnedState::new(admissions.clone()),
        };
        (forked, pinned)
    }

    /// Whether this subsystem is backed by a cold checkpoint's naming pages.
    pub fn has_cold_naming(&self) -> bool {
        self.state.read().cold_naming.is_some()
    }

    /// Raises `branch`'s naming retention floor: its name index atomically
    /// compacts to retained bases and preserves retired-name continuity.
    ///
    /// Complexity: O(branch rows) once per raise.
    pub fn advance_naming_retention_floor(
        &self,
        branch: &BranchId,
        floor: CommitId,
    ) -> Result<(), PersistentNameIndexDenial> {
        let mut state = self.state.write();
        if let Some(BranchNameIndex::Current(index)) = state.name_indexes.by_branch.get_mut(branch)
        {
            index.advance_retention_floor(floor)?;
            state.bump_branch_generation(branch);
        }
        Ok(())
    }
}
