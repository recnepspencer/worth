use crate::identity::data::{EntityId, KindId};
use crate::storage::data::RelationReadRecord;
use crate::storage::overlay::PartitionState;

use super::{AdjacencySet, SparseAdjacencyTable};

/// Which side of a relation an adjacency traversal walks.
///
/// Direction is data, not control flow: both bounded readers pick their table
/// and their endpoint test from the same value, so the outgoing and incoming
/// paths cannot drift apart in their charge schedule or their filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AdjacencyDirection {
    Outgoing,
    Incoming,
}

impl AdjacencyDirection {
    pub(crate) fn table(self, partition: &PartitionState) -> &SparseAdjacencyTable {
        match self {
            Self::Outgoing => &partition.adjacency,
            Self::Incoming => &partition.reverse_adjacency,
        }
    }

    pub(crate) fn matches_endpoint(self, record: &RelationReadRecord, entity_id: EntityId) -> bool {
        match self {
            Self::Outgoing => record.source == entity_id,
            Self::Incoming => record.target == entity_id,
        }
    }
}

/// Which generation of a kind's adjacency a traversal reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AdjacencyKindBasis {
    Current,
    Historical,
}

impl AdjacencyKindBasis {
    pub(crate) fn of_current_version(current_version: bool) -> Self {
        if current_version {
            Self::Current
        } else {
            Self::Historical
        }
    }
}

impl AdjacencySet {
    /// Lend one kind's adjacency ids without copying them.
    ///
    /// The view borrows the partition, which borrows the caller's pinned
    /// edition. A bounded reader must consume this lease under its work budget
    /// rather than materializing it first: the whole point of a bound is that
    /// the fanout is never fully touched.
    pub(crate) fn kind_ids(
        &self,
        basis: AdjacencyKindBasis,
        kind_id: KindId,
    ) -> super::AdjacencyIds<'_> {
        match basis {
            AdjacencyKindBasis::Current => self.current_kind_ids(kind_id),
            AdjacencyKindBasis::Historical => self.historical_kind_ids(kind_id),
        }
    }
}
