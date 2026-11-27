//! One held name's derived history: its liveness timeline and the forward
//! lineage edges recorded at each commit.
//!
//! `NameEntry` is deliberately not `Clone`: its timeline grows with every
//! commit that touches the name, so a copy is O(history). The index journals
//! a touched entry by its history lengths alone (`mark` / `truncate_to`), and
//! a branch fork copies the index only through its named fork entry point.

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};
use crate::contracts::lineage::data::naming::{PersistentNameBirthParts, PersistentNameIdentity};

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

/// Derived per-name history, rebuilt solely from committed lineage batches.
#[derive(Debug, PartialEq, Eq)]
pub struct NameEntry {
    /// The canonical birth or group scheme the name was born from.
    origin: NameOrigin,
    /// Strictly ascending commits from birth; each state holds until the
    /// next entry.
    timeline: Vec<(CommitId, NameState)>,
    /// Strictly ascending commits; successors sorted and deduplicated.
    forward_edges: Vec<(CommitId, Vec<PersistentNameKey>)>,
}

impl NameEntry {
    pub(crate) fn born(born_at: CommitId, origin: NameOrigin, live: LiveName) -> Self {
        Self {
            origin,
            timeline: vec![(born_at, NameState::Live(live))],
            forward_edges: Vec::new(),
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
        self.timeline[0].0
    }

    /// The state in force at `basis` and the commit it began, or `None` when
    /// the name was born after `basis`.
    ///
    /// Complexity: O(log t) in this name's own timeline length.
    pub(crate) fn state_at(&self, basis: CommitId) -> Option<(CommitId, &NameState)> {
        let after = self
            .timeline
            .partition_point(|(commit, _)| *commit <= basis);
        let (commit, state) = self.timeline.get(after.checked_sub(1)?)?;
        Some((*commit, state))
    }

    /// The latest state and commit, as of the newest ingested commit.
    pub(crate) fn last_state(&self) -> (CommitId, &NameState) {
        let (commit, state) = self
            .timeline
            .last()
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
    /// Complexity: O(log t) in this name's own timeline length.
    pub(crate) fn lineage_ended_at(&self, ended_at: CommitId) -> Option<LineageId> {
        let before = self
            .timeline
            .partition_point(|(commit, _)| *commit < ended_at);
        self.timeline[..before]
            .iter()
            .rev()
            .find_map(|(_, state)| match state {
                NameState::Live(live) => live.target.lineage(),
                NameState::Ended => None,
            })
    }

    /// Whether `lineage` held this name at some commit at or before `basis`.
    ///
    /// Complexity: O(t) in this name's own timeline length.
    pub(crate) fn binds_lineage_by(&self, lineage: LineageId, basis: CommitId) -> bool {
        self.timeline.iter().any(|(commit, state)| {
            *commit <= basis
                && matches!(state, NameState::Live(live) if live.target.lineage() == Some(lineage))
        })
    }

    /// Forward lineage edges recorded at exactly `commit`.
    ///
    /// Complexity: O(log e) in this name's own edge-commit count.
    pub(crate) fn edges_at(&self, commit: CommitId) -> &[PersistentNameKey] {
        self.forward_edges
            .binary_search_by_key(&commit, |(at, _)| *at)
            .map_or(&[], |position| self.forward_edges[position].1.as_slice())
    }

    /// Sets the state from `commit` on; a later write in the same commit
    /// replaces the earlier one.
    pub(crate) fn set_state(&mut self, commit: CommitId, state: NameState) {
        match self.timeline.last_mut() {
            Some((at, last)) if *at == commit => *last = state,
            _ => self.timeline.push((commit, state)),
        }
    }

    /// A branch fork's copy of this history (see
    /// `PersistentNameBranchIndex::duplicate_for_branch_fork`).
    pub(super) fn duplicate_for_branch_fork(&self) -> Self {
        Self {
            origin: self.origin.clone(),
            timeline: self.timeline.clone(),
            forward_edges: self.forward_edges.clone(),
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
        match self.forward_edges.last_mut() {
            Some((at, successors)) if *at == commit => {
                if let Err(position) = successors.binary_search(&successor) {
                    successors.insert(position, successor);
                }
            }
            _ => self.forward_edges.push((commit, vec![successor])),
        }
    }
}
