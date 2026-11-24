//! One installed group scheme's committed transition.

use crate::contracts::aggregate_identity::transition::group::model::AggregateGroupScheme;
use crate::contracts::aggregate_identity::transition::group::transition_result::AggregateGroupTransition;

/// Which authority produced a committed group transition.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum GroupTransitionKind {
    /// The group naming authority named a declared (or member-retirement
    /// derived) partition of the scheme's changed scope.
    Partition,
    /// A history selection restored the scheme's groups to their exact state
    /// at the selected commit.
    Selection,
}

impl GroupTransitionKind {
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::Partition => 1,
            Self::Selection => 2,
        }
    }

    pub(crate) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::Partition),
            2 => Some(Self::Selection),
            _ => None,
        }
    }
}

/// The complete group naming outcome one commit published for one scheme:
/// every touched next-state group, every retired group and the canonical
/// group lineage events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedGroupTransition {
    pub kind: GroupTransitionKind,
    pub scheme: AggregateGroupScheme,
    pub transition: AggregateGroupTransition,
}

impl CommittedGroupTransition {
    pub fn scheme_id(&self) -> &str {
        self.scheme.id().as_str()
    }
}
