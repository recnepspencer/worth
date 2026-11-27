//! One held name's derived history: its liveness timeline and the forward
//! lineage edges recorded at each commit.
//!
//! `NameEntry` is deliberately not `Clone`: its timeline grows with every
//! commit that touches the name, so an unintended copy is fenced at compile
//! time. Internally, its histories use persistent `im::OrdMap` tables and
//! `Arc`-shared successor lists so both `duplicate_for_branch_fork` and
//! single-descent commit-key predecessor lookups stay strictly `O(1)` / `O(log n)`.

use std::sync::Arc;

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};
use crate::contracts::lineage::data::naming::{PersistentNameBirthParts, PersistentNameIdentity};

/// Exact `O(log n)` predecessor lookup (`k <= key`) on `im::OrdMap`.
///
/// Uses `range(..=key).next_back()` rather than `im::OrdMap::get_prev`, which
/// drops promoted parent separators after a B-tree split in `im 15.1.0`.
pub(super) fn ordmap_prev<'a, K: Ord, V>(
    map: &'a im::OrdMap<K, V>,
    key: &K,
) -> Option<(&'a K, &'a V)> {
    map.range(..=key).next_back()
}

/// What a name designates from one commit until its next timeline entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NameState {
    Live(LiveName),
    Ended,
}

/// A name's declared carriers: a persistent set, so a timeline state that
/// keeps or edits its predecessor's carriers shares every unchanged node
/// instead of copying the set.
pub(crate) type Carriers = im::OrdSet<PersistentNameIdentity>;

/// A live name's record and its declared carriers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveName {
    pub(crate) target: PersistentNameTarget,
    pub(crate) carriers: Carriers,
}

/// What a name was born from: an entity or relation name's canonical birth
/// parts from its `Create` event, or a group's installed scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NameOrigin {
    Birth(PersistentNameBirthParts),
    Group { scheme_id: Option<String> },
}

crate::lineage::clone_fences::assert_not_clone!(NameEntry);

/// One record's persistent binding-epoch history, keyed by `(bound_at, lineage)`
/// with its optional retirement/transition commit `ended_at`.
///
/// Backed by `im::OrdMap` so fork copy-on-write is `O(1)` and historical
/// predecessor lookups descend the tree once in `O(log e)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct HolderEpochs(pub(crate) im::OrdMap<(CommitId, LineageId), Option<CommitId>>);

impl HolderEpochs {
    pub(crate) fn latest(&self) -> Option<(CommitId, LineageId, Option<CommitId>)> {
        self.0
            .get_max()
            .map(|((bound_at, lineage), ended_at)| (*bound_at, *lineage, *ended_at))
    }

    /// The lineage bound to this holder at `basis` in one `O(log e)` descent.
    pub(crate) fn lineage_at(&self, basis: CommitId) -> Option<LineageId> {
        ordmap_prev(&self.0, &(basis, LineageId(u64::MAX))).map(|((_, lineage), _)| *lineage)
    }

    pub(crate) fn insert_epoch(&mut self, bound_at: CommitId, lineage: LineageId) {
        self.0.insert((bound_at, lineage), None);
    }

    pub(crate) fn remove_epoch(&mut self, key: &(CommitId, LineageId)) {
        self.0.remove(key);
    }

    /// Inspects epochs immutably in reverse commit order to locate `lineage`'s
    /// latest binding key before mutating `ended_at`.
    pub(crate) fn find_latest_key(&self, lineage: LineageId) -> Option<(CommitId, LineageId)> {
        self.0
            .iter()
            .rev()
            .find(|((_, l), _)| *l == lineage)
            .map(|(k, _)| *k)
    }

    pub(crate) fn set_ended_at(
        &mut self,
        key: &(CommitId, LineageId),
        ended_at: Option<CommitId>,
    ) -> Option<Option<CommitId>> {
        let slot = self.0.get_mut(key)?;
        let prior = *slot;
        *slot = ended_at;
        Some(prior)
    }
}

/// Derived per-name history, rebuilt solely from committed lineage batches.
///
/// Backed by persistent `im::OrdMap` tables and `Arc`-shared successor vectors
/// so copy-on-write fork duplication is `O(1)` and commit-key predecessor
/// lookups take one `O(log n)` descent while `NameEntry` remains `!Clone`.
#[derive(Debug, PartialEq, Eq)]
pub struct NameEntry {
    /// The canonical birth or group scheme the name was born from.
    pub(super) origin: NameOrigin,
    /// The commit the name was born at; retention compaction never moves it.
    pub(super) born: CommitId,
    /// Strictly ascending commits from birth; each state holds until the
    /// next entry.
    pub(super) timeline: im::OrdMap<CommitId, NameState>,
    /// Strictly ascending commits; successors sorted, deduplicated, and
    /// `Arc`-shared so fork writes never deep-copy historical wide splits.
    pub(super) forward_edges: im::OrdMap<CommitId, Arc<Vec<PersistentNameKey>>>,
    /// Earliest commit at which each `LineageId` held this name; maintained on
    /// every live write and preserved across retention compaction so
    /// `binds_lineage_by` is strictly `O(log L)` without scanning `timeline`.
    pub(crate) lineage_commits: im::OrdMap<LineageId, CommitId>,
}

impl NameEntry {
    pub(crate) fn born(born_at: CommitId, origin: NameOrigin, live: LiveName) -> Self {
        let mut lineage_commits = im::OrdMap::new();
        if let Some(lineage) = live.target.lineage() {
            lineage_commits.insert(lineage, born_at);
        }
        let mut timeline = im::OrdMap::new();
        timeline.insert(born_at, NameState::Live(live));
        Self {
            origin,
            born: born_at,
            timeline,
            forward_edges: im::OrdMap::new(),
            lineage_commits,
        }
    }

    /// The canonical birth discriminator; a name born without one has no
    /// side, exactly like a group.
    pub fn side(&self) -> Option<&[u8]> {
        self.birth()
            .map(PersistentNameBirthParts::canonical_discriminator)
            .filter(|canonical| !canonical.is_empty())
    }

    /// The installed scheme the name was born under.
    pub(crate) fn scheme_id(&self) -> Option<&str> {
        match &self.origin {
            NameOrigin::Birth(parts) => Some(parts.scheme_id()),
            NameOrigin::Group { scheme_id } => scheme_id.as_deref(),
        }
    }

    /// The canonical birth parts an entity or relation name was minted
    /// from; groups carry none.
    pub(crate) fn birth(&self) -> Option<&PersistentNameBirthParts> {
        match &self.origin {
            NameOrigin::Birth(parts) => Some(parts),
            NameOrigin::Group { .. } => None,
        }
    }

    /// The commit this name was born at.
    pub(crate) fn born_at(&self) -> CommitId {
        self.born
    }

    /// The state in force at `basis` and the commit it began, or `None` when
    /// the name was born after `basis`.
    ///
    /// Complexity: single `O(log t)` B-tree predecessor descent.
    pub(crate) fn state_at(&self, basis: CommitId) -> Option<(CommitId, &NameState)> {
        ordmap_prev(&self.timeline, &basis).map(|(commit, state)| (*commit, state))
    }

    /// The latest state and commit, as of the newest ingested commit.
    pub(crate) fn last_state(&self) -> (CommitId, &NameState) {
        let (commit, state) = self
            .timeline
            .get_max()
            .expect("a name entry always holds its birth state");
        (*commit, state)
    }

    /// The latest state, as of the newest ingested commit.
    pub(crate) fn current(&self) -> &NameState {
        self.last_state().1
    }

    pub(crate) fn current_live(&self) -> Option<&LiveName> {
        match self.current() {
            NameState::Live(live) => Some(live),
            NameState::Ended => None,
        }
    }

    /// The most recent `LineageId` held by this name across its live epochs,
    /// or `None` for groups.
    pub(crate) fn last_live_lineage(&self) -> Option<LineageId> {
        self.timeline
            .iter()
            .rev()
            .find_map(|(_, state)| match state {
                NameState::Live(live) => match live.target {
                    PersistentNameTarget::Entity { lineage, .. }
                    | PersistentNameTarget::Relation { lineage, .. } => Some(lineage),
                    PersistentNameTarget::Group { .. } => None,
                },
                NameState::Ended => None,
            })
    }

    /// The lineage this name held until it ended at `ended_at`; `None` for
    /// groups.
    ///
    /// Complexity: single `O(log t)` B-tree predecessor descent.
    pub(crate) fn lineage_ended_at(&self, ended_at: CommitId) -> Option<LineageId> {
        let prev_limit = ended_at.0.checked_sub(1)?;
        match ordmap_prev(&self.timeline, &CommitId(prev_limit)) {
            Some((_, NameState::Live(live))) => live.target.lineage(),
            _ => None,
        }
    }

    /// Whether `lineage` held this name at some commit at or before `basis`.
    ///
    /// Complexity: O(log L) in the number of lineages that ever held this name.
    pub(crate) fn binds_lineage_by(&self, lineage: LineageId, basis: CommitId) -> bool {
        self.lineage_commits
            .get(&lineage)
            .is_some_and(|first| *first <= basis)
    }

    /// Forward lineage edges recorded at exactly `commit`.
    ///
    /// Complexity: single `O(log e)` B-tree lookup.
    pub(crate) fn edges_at(&self, commit: CommitId) -> &[PersistentNameKey] {
        self.forward_edges
            .get(&commit)
            .map_or(&[], |successors| successors.as_slice())
    }

    /// Sets the state from `commit` on; a later write in the same commit
    /// replaces the earlier one. Returns `Some(lineage)` when a new lineage
    /// binding was inserted into `lineage_commits` so rollback stays O(batch).
    pub(crate) fn set_state(&mut self, commit: CommitId, state: NameState) -> Option<LineageId> {
        let inserted = if let NameState::Live(live) = &state
            && let Some(lineage) = live.target.lineage()
            && !self.lineage_commits.contains_key(&lineage)
        {
            self.lineage_commits.insert(lineage, commit);
            Some(lineage)
        } else {
            None
        };
        self.timeline.insert(commit, state);
        inserted
    }

    /// Removes one lineage binding inserted by the rolling-back batch.
    pub(super) fn remove_lineage_commit(&mut self, lineage: LineageId) {
        self.lineage_commits.remove(&lineage);
    }

    /// Copy-on-write duplicate of this history when a shared `TieredMap` entry
    /// is mutated or materialized for an audit/retention pass.
    ///
    /// Complexity: O(1) structural sharing via persistent `im` handles.
    pub(super) fn duplicate_for_branch_fork(&self) -> Self {
        Self {
            origin: self.origin.clone(),
            born: self.born,
            timeline: self.timeline.clone(),
            forward_edges: self.forward_edges.clone(),
            lineage_commits: self.lineage_commits.clone(),
        }
    }

    /// Drops the timeline state and forward-edge row recorded by the rolling-
    /// back batch at `commit` (which is strictly above pre-batch head).
    ///
    /// Complexity: O(log t + log e).
    pub(super) fn rollback_commit(&mut self, commit: CommitId) {
        self.timeline.remove(&commit);
        self.forward_edges.remove(&commit);
    }

    /// Appends `successor` to `commit`'s forward edges. Inspects `commit`
    /// immutably first and shares each commit's successor list behind `Arc`, so
    /// fork writes never clone historical successor lists.
    pub(crate) fn add_edge(&mut self, commit: CommitId, successor: PersistentNameKey) {
        if let Some(existing) = self.forward_edges.get(&commit) {
            if existing.binary_search(&successor).is_ok() {
                return;
            }
            if let Some(slot) = self.forward_edges.get_mut(&commit) {
                let successors = Arc::make_mut(slot);
                if let Err(position) = successors.binary_search(&successor) {
                    successors.insert(position, successor);
                }
            }
        } else {
            self.forward_edges.insert(commit, Arc::new(vec![successor]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineage::index::member_groups::{MemberTenure, MemberTenures};
    use crate::lineage::index::retention::{compact_entry, compact_tenures, first_retained_commit};
    use worth_foundational::facade::AggregateIdentity;

    #[test]
    fn predecessor_lookups_survive_btree_splits_at_promoted_separator_gaps() {
        let g = |n: u8| AggregateIdentity::from_bytes([n; 32]);
        let live = |n: u8| {
            NameState::Live(LiveName {
                target: PersistentNameTarget::Group { group: g(n) },
                carriers: Carriers::new(),
            })
        };
        let mut entry = NameEntry::born(
            CommitId(2),
            NameOrigin::Group {
                scheme_id: Some("s".into()),
            },
            LiveName {
                target: PersistentNameTarget::Group { group: g(1) },
                carriers: Carriers::new(),
            },
        );
        let mut epochs = HolderEpochs::default();
        let mut tenure = MemberTenure::new();
        for i in 1u64..=130 {
            let c = CommitId(i * 2);
            if i > 1 {
                entry.set_state(c, live(i as u8));
            }
            epochs.insert_epoch(c, LineageId(i));
            tenure.insert(c, Some(g(i as u8)));
        }
        assert!(entry.state_at(CommitId(1)).is_none());
        assert!(epochs.lineage_at(CommitId(1)).is_none());
        assert!(ordmap_prev(&tenure, &CommitId(1)).is_none());
        // Verify every exact commit AND every odd gap (including 67 after promoted key 66).
        for probe in 2u64..=261 {
            let expected_i = (probe / 2).min(130);
            let expected_c = CommitId(expected_i * 2);
            let (got_c, _) = entry
                .state_at(CommitId(probe))
                .expect("predecessor must never vanish across B-tree split");
            assert_eq!(got_c, expected_c, "NameEntry::state_at({probe})");
            assert_eq!(
                epochs.lineage_at(CommitId(probe)),
                Some(LineageId(expected_i)),
                "HolderEpochs::lineage_at({probe})"
            );
            assert_eq!(
                ordmap_prev(&tenure, &CommitId(probe)).map(|(c, _)| *c),
                Some(expected_c),
                "MemberTenure prev({probe})"
            );
            assert_eq!(
                first_retained_commit(&entry.timeline, CommitId(probe)),
                Some(expected_c),
                "first_retained_commit({probe})"
            );
        }
        // Unchanged floor before earliest commit returns None without rebuilding.
        assert!(compact_entry(&entry, CommitId(1)).is_none());
        let mut tenures = MemberTenures::new();
        tenures.insert("s".into(), tenure);
        assert!(compact_tenures(&tenures, CommitId(1)).is_none());
    }
}
