use std::fmt;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;

/// Typed refusal to ingest a committed batch into the derived name index.
///
/// A refusal means the supplied log is not canonical committed history; the
/// index is left exactly as it was before the refused batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentNameIndexDenial {
    /// Batches must arrive in strictly ascending commit order.
    NonMonotonicCommit { head: CommitId, next: CommitId },
    /// A lineage event belongs to another branch or another commit.
    ForeignEvent {
        expected_branch: BranchId,
        expected_commit: CommitId,
        event_id: u64,
    },
    /// A `Create` event targets a lineage with no bound record.
    MissingHolder { lineage: LineageId },
    /// One lineage was bound to two different records.
    ConflictingHolder { lineage: LineageId },
    /// A birth names an identity the index already holds.
    DuplicateBirth { name: PersistentNameKey },
    /// A transition does not fit the name's derived state.
    InvalidTransition {
        name: PersistentNameKey,
        detail: &'static str,
    },
}

impl fmt::Display for PersistentNameIndexDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonMonotonicCommit { head, next } => write!(
                formatter,
                "name index batch {next:?} does not follow head {head:?}"
            ),
            Self::ForeignEvent {
                expected_branch,
                expected_commit,
                event_id,
            } => write!(
                formatter,
                "lineage event {event_id} is not from {expected_branch:?} at {expected_commit:?}"
            ),
            Self::MissingHolder { lineage } => {
                write!(formatter, "created lineage {lineage:?} has no bound record")
            }
            Self::ConflictingHolder { lineage } => {
                write!(formatter, "lineage {lineage:?} is bound to two records")
            }
            Self::DuplicateBirth { name } => {
                write!(formatter, "persistent name {name:?} is born twice")
            }
            Self::InvalidTransition { name, detail } => {
                write!(formatter, "persistent name {name:?}: {detail}")
            }
        }
    }
}

impl std::error::Error for PersistentNameIndexDenial {}
