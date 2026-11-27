//! One held name's derived history: its liveness timeline and the forward
//! lineage edges recorded at each commit.
//!
//! `NameEntry` is deliberately not `Clone`: its timeline grows with every
//! commit that touches the name, so a copy is O(history). The index journals
//! a touched entry by its history lengths alone (`mark` / `truncate_to`), and
//! a branch fork shares entries copy-on-write via `TieredMap`.

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};
use crate::contracts::lineage::data::naming::{PersistentNameBirthParts, PersistentNameIdentity};

/// Binary search partition point on a sorted `im::Vector`.
///
/// Complexity: O(log n) indexed probes, zero allocation.
pub(super) fn partition_point_im<T: Clone>(
    vec: &im::Vector<T>,
    mut pred: impl FnMut(&T) -> bool,
) -> usize {
    let (mut left, mut right) = (0, vec.len());
    while left < right {
        let mid = left + (right - left) / 2;
        if pred(&vec[mid]) {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    left
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

/// One record-to-lineage binding epoch, including its retirement/transition
/// commit when the lineage ended so cold historical forks never resurrect
/// retired unnamed lineages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HolderEpoch {
    pub(crate) bound_at: CommitId,
    pub(crate) lineage: LineageId,
    pub(crate) ended_at: Option<CommitId>,
}

/// Derived per-name history, rebuilt solely from committed lineage batches.
///
/// Backed by structural persistent `im` collections so copy-on-write fork
/// duplication is strictly `O(1)` regardless of prior history length while
/// `NameEntry` itself remains `!Clone`.
#[derive(Debug, PartialEq, Eq)]
pub struct NameEntry {
    /// The canonical birth or group scheme the name was born from.
    pub(super) origin: NameOrigin,
    /// The commit the name was born at; retention compaction never moves it.
    pub(super) born: CommitId,
    /// Strictly ascending commits from birth; each state holds until the
    /// next entry.
    pub(super) timeline: im::Vector<(CommitId, NameState)>,
    /// Strictly ascending commits; successors sorted and deduplicated.
    pub(super) forward_edges: im::Vector<(CommitId, Vec<PersistentNameKey>)>,
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
        Self {
            origin,
            born: born_at,
            timeline: im::vector![(born_at, NameState::Live(live))],
            forward_edges: im::Vector::new(),
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
    /// Complexity: O(log t) in this name's own timeline length.
    pub(crate) fn state_at(&self, basis: CommitId) -> Option<(CommitId, &NameState)> {
        let after = partition_point_im(&self.timeline, |(commit, _)| *commit <= basis);
        let (commit, state) = self.timeline.get(after.checked_sub(1)?)?;
        Some((*commit, state))
    }

    /// The latest state and commit, as of the newest ingested commit.
    pub(crate) fn last_state(&self) -> (CommitId, &NameState) {
        let (commit, state) = self
            .timeline
            .back()
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
    /// Complexity: O(log t) binary search plus O(1) immediate predecessor check.
    pub(crate) fn lineage_ended_at(&self, ended_at: CommitId) -> Option<LineageId> {
        let before = partition_point_im(&self.timeline, |(commit, _)| *commit < ended_at);
        let prev = before.checked_sub(1)?;
        match self.timeline.get(prev) {
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
    /// Complexity: O(log e) in this name's own edge-commit count.
    pub(crate) fn edges_at(&self, commit: CommitId) -> &[PersistentNameKey] {
        let position = partition_point_im(&self.forward_edges, |(at, _)| *at < commit);
        match self.forward_edges.get(position) {
            Some((at, successors)) if *at == commit => successors.as_slice(),
            _ => &[],
        }
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
        match self.timeline.back_mut() {
            Some((at, last)) if *at == commit => *last = state,
            _ => self.timeline.push_back((commit, state)),
        }
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

    /// How long this name's histories are now: what rollback truncates to.
    pub(super) fn mark(&self) -> super::journal::EntryMark {
        super::journal::EntryMark {
            timeline: self.timeline.len(),
            edges: self.forward_edges.len(),
        }
    }

    /// Drops every timeline state and edge row recorded after `mark`.
    ///
    /// Complexity: O(rows dropped), the rows one batch appended.
    pub(super) fn truncate_to(&mut self, mark: super::journal::EntryMark) {
        self.timeline.truncate(mark.timeline);
        self.forward_edges.truncate(mark.edges);
    }

    pub(crate) fn add_edge(&mut self, commit: CommitId, successor: PersistentNameKey) {
        match self.forward_edges.back_mut() {
            Some((at, successors)) if *at == commit => {
                if let Err(position) = successors.binary_search(&successor) {
                    successors.insert(position, successor);
                }
            }
            _ => self.forward_edges.push_back((commit, vec![successor])),
        }
    }
}
