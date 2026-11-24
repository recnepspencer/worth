//! The bytes or original member location carried by a checkpoint capture.
use crate::durability::medium::RelationalDurableMemberLocator;

/// One content-addressed member selected by a checkpoint capture. A carried
/// member already belongs to the selected durable checkpoint. Publication must
/// preserve its membership along with the new manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalCheckpointMember {
    New {
        locator: RelationalDurableMemberLocator,
        bytes: worth_execution::ExecutionImmutableBytes,
    },
    Carried {
        locator: RelationalDurableMemberLocator,
    },
}

impl RelationalCheckpointMember {
    pub fn locator(&self) -> RelationalDurableMemberLocator {
        match self {
            Self::New { locator, .. } | Self::Carried { locator } => *locator,
        }
    }
}
