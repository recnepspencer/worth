use std::fmt;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause;

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
    /// A cold naming page the index needed is missing or damaged; the index
    /// refuses rather than reading the page as absent rows.
    StorageUnavailable(RelationalStorageUnavailableCause),
    /// The checkpoint this index was reopened from recorded it unavailable;
    /// it stays unavailable until canonical history rebuilds it.
    CheckpointedUnavailable,
    /// The queried commit basis falls strictly below the branch's compacted
    /// retention floor.
    BasisPruned {
        basis: CommitId,
        retention_floor: CommitId,
    },
    /// A requested retention floor exceeds the branch's current head commit.
    RetentionFloorExceedsHead {
        floor: CommitId,
        head: Option<CommitId>,
    },
}

impl From<RelationalStorageUnavailableCause> for PersistentNameIndexDenial {
    fn from(cause: RelationalStorageUnavailableCause) -> Self {
        Self::StorageUnavailable(cause)
    }
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
            Self::StorageUnavailable(cause) => {
                write!(formatter, "a cold naming page is unavailable: {cause:?}")
            }
            Self::CheckpointedUnavailable => {
                formatter.write_str("the reopened checkpoint recorded this index unavailable")
            }
            Self::BasisPruned {
                basis,
                retention_floor,
            } => write!(
                formatter,
                "basis {basis:?} falls below persistent-name retention floor {retention_floor:?}"
            ),
            Self::RetentionFloorExceedsHead { floor, head } => write!(
                formatter,
                "requested persistent-name retention floor {floor:?} exceeds branch head {head:?}"
            ),
        }
    }
}

impl std::error::Error for PersistentNameIndexDenial {}
