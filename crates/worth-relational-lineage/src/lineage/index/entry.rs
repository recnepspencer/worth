//! One held name's derived history: its liveness timeline and the forward
//! lineage edges recorded at each commit.

use std::collections::BTreeSet;

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameTarget};
use crate::contracts::lineage::data::naming::PersistentNameIdentity;

/// What a name designates from one commit until its next timeline entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NameState {
    Live(LiveName),
    Ended,
}

/// A live name's record and its declared carriers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveName {
    pub(crate) target: PersistentNameTarget,
    pub(crate) carriers: BTreeSet<PersistentNameIdentity>,
}

/// Derived per-name history, rebuilt solely from committed lineage batches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameEntry {
    /// Canonical birth discriminator; groups carry none.
    side: Option<Vec<u8>>,
    /// The installed scheme the name was born under; groups carry none.
    scheme_id: Option<String>,
    /// Strictly ascending commits from birth; each state holds until the
    /// next entry.
    timeline: Vec<(CommitId, NameState)>,
    /// Strictly ascending commits; successors sorted and deduplicated.
    forward_edges: Vec<(CommitId, Vec<PersistentNameKey>)>,
}

impl NameEntry {
    pub(crate) fn born(
        born_at: CommitId,
        side: Option<Vec<u8>>,
        scheme_id: Option<String>,
        live: LiveName,
    ) -> Self {
        Self {
            side,
            scheme_id,
            timeline: vec![(born_at, NameState::Live(live))],
            forward_edges: Vec::new(),
        }
    }

    pub fn side(&self) -> Option<&[u8]> {
        self.side.as_deref()
    }

    pub(crate) fn scheme_id(&self) -> Option<&str> {
        self.scheme_id.as_deref()
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
