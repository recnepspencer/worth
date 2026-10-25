use std::sync::Arc;

use crate::durability::medium::RelationalDurableMemberLocator;

/// Placement of the member set of one native checkpoint. The member framing
/// and manifest are identical in both placements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalCheckpointArtifactPolicy {
    Embedded,
    ExternalMembers,
}

/// One content-addressed member selected by a checkpoint capture. A carried
/// member already belongs to the selected durable checkpoint. Publication must
/// preserve its membership along with the new manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalCheckpointMember {
    New {
        locator: RelationalDurableMemberLocator,
        bytes: Arc<[u8]>,
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
