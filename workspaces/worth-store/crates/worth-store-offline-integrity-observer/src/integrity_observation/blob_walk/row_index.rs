//! The selected rows a check asks for, and what it means that none answers.

use std::collections::BTreeMap;

use super::super::blob_record::{BlobFact, FrameKind, FrameRole};
use super::coverage::Coverage;
use super::graph::dependency_uncertainty;
use super::proof::Proof;
use super::{Outcome, Selected};

/// Selected rows by record and declarations by session, with what the walk
/// could not see. Unread is not absent: a row that no lookup finds contradicts
/// a claim only when the walk saw every row that could be it.
pub(super) struct RowIndex {
    /// The first row of each record.
    pub(super) records: BTreeMap<[u8; 24], usize>,
    /// The first declaration of each session.
    pub(super) sessions: BTreeMap<[u8; 16], usize>,
    /// Set when the walk did not visit every routed record.
    unvisited: Option<Outcome>,
    /// The undecided rows whose frame could not be read, each with the kind
    /// that its route or its first bytes declare. Such a row answers for its
    /// record, and it may be a row that a check looks for by what the frame
    /// says only if its kind lets it, or if nothing says its kind.
    unread: Vec<(Option<FrameKind>, Outcome)>,
}

impl RowIndex {
    pub(super) fn new(selected: &[Selected], coverage: &Coverage) -> Self {
        let mut records = BTreeMap::new();
        let mut sessions = BTreeMap::new();
        for (index, row) in selected.iter().enumerate() {
            records.entry(row.record).or_insert(index);
            if let Some(BlobFact::Declaration { session, .. }) = row.fact.as_ref() {
                sessions.entry(*session).or_insert(index);
            }
        }
        let unread = selected.iter().filter(|row| row.fact.is_none());
        Self {
            records,
            sessions,
            unvisited: coverage.unvisited().cloned(),
            unread: unread
                .filter_map(|row| Some((row.kind, dependency_uncertainty(row)?)))
                .collect(),
        }
    }

    /// The row of `record`, or the verdict that its absence gives a check.
    /// Every record the walk visited has a row, read or not.
    pub(super) fn record(&self, record: &[u8; 24]) -> Result<usize, Proof> {
        let row = self.records.get(record).copied();
        row.ok_or_else(|| absent(self.unvisited.as_ref()))
    }

    /// The verdict on a claim that none of `records` is in the store any
    /// longer.
    pub(super) fn all_gone(&self, records: &[[u8; 24]]) -> Proof {
        let gone = |record| match self.record(record) {
            Ok(_) => Proof::Contradicted,
            Err(Proof::Contradicted) => Proof::Holds,
            Err(unvisited) => unvisited,
        };
        let all = |proof: Proof, record| proof.and(gone(record));
        records.iter().fold(Proof::Holds, all)
    }

    /// The declaration of `session`, or the verdict that its absence gives a
    /// check.
    pub(super) fn declaration(&self, session: &[u8; 16]) -> Result<usize, Proof> {
        let row = self.sessions.get(session).copied();
        row.ok_or_else(|| self.none_found(FrameRole::Declaration))
    }

    /// The verdict on a claim that needs a row known only by what its frame
    /// says, when no frame that was read is that row.
    pub(super) fn none_found(&self, role: FrameRole) -> Proof {
        absent(self.unlocated(role))
    }

    /// The outcome of a row that may be the `role` a check seeks, selected
    /// where the walk could not see: a frame that could not be read and is of
    /// a kind that may be that row, or of no kind that the walk could tell,
    /// else a record that the walk did not visit.
    pub(super) fn unlocated(&self, role: FrameRole) -> Option<&Outcome> {
        let may_be = |kind: &Option<FrameKind>| kind.is_none_or(|kind| kind.may_be(role));
        let unread = self.unread.iter().find(|(kind, _)| may_be(kind));
        unread
            .map(|(_, outcome)| outcome)
            .or(self.unvisited.as_ref())
    }

    /// Rows that share a record with a row of another frame, and declarations
    /// that share a session with a declaration of another record.
    pub(super) fn duplicates(&self, selected: &[Selected]) -> Vec<usize> {
        let mut duplicates = Vec::new();
        for (index, row) in selected.iter().enumerate() {
            let first = self.records[&row.record];
            if first != index && selected[first].fact != row.fact {
                duplicates.extend([first, index]);
            }
            let Some(BlobFact::Declaration { session, .. }) = row.fact.as_ref() else {
                continue;
            };
            let first = self.sessions[session];
            if first != index && selected[first].record != row.record {
                duplicates.extend([first, index]);
            }
        }
        duplicates
    }
}

fn absent(unseen: Option<&Outcome>) -> Proof {
    unseen
        .cloned()
        .map_or(Proof::Contradicted, Proof::Undecided)
}
