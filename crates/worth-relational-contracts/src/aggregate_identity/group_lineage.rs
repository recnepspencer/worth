//! Canonical group lineage records, independent of native slot order.
//!
//! Unlike the stored edge grammar read by `stored_lineage`, these records carry
//! no `CommitId` and no native coordinate: the commit envelope supplies the
//! occurrence when they attach to the canonical lineage authority.
use crate::history::data::coordinates::CommitId;
use worth_foundational::facade::{AggregateIdentity, AggregateLineageKind};

/// One group birth, revival, split, merge, or retirement.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum AggregateGroupLineageEvent {
    /// A fresh group name minted from its birth anchor.
    Born { group: AggregateIdentity },
    /// A retired group name re-formed around its original anchor.
    Revived { group: AggregateIdentity },
    /// `fragment` received members from `source`, which did not keep them.
    SplitFrom {
        fragment: AggregateIdentity,
        source: AggregateIdentity,
    },
    /// `absorbed` retired into the group that `keeper` names.
    MergedInto {
        absorbed: AggregateIdentity,
        keeper: AggregateIdentity,
    },
    /// A prior live group name retired in this transition.
    Retired { group: AggregateIdentity },
}

impl AggregateGroupLineageEvent {
    /// The stored edge kind this event projects to, when it is an edge.
    pub const fn edge_kind(self) -> Option<AggregateLineageKind> {
        match self {
            Self::SplitFrom { .. } => Some(AggregateLineageKind::SplitFrom),
            Self::MergedInto { .. } => Some(AggregateLineageKind::MergedInto),
            Self::Born { .. } | Self::Revived { .. } | Self::Retired { .. } => None,
        }
    }
}

/// One group name's retained lifecycle at a selected commit.
pub struct AggregateGroupHistory {
    pub retired: Option<CommitId>,
    pub events: Vec<(CommitId, AggregateGroupLineageEvent)>,
}
