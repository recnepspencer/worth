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
