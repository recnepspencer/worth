use std::sync::Arc;

use crate::durability::medium::RelationalDurableMemberLocator;

/// Placement of the member set of one native checkpoint. The member framing
/// and manifest are identical in both placements.
#[derive(Debug, Clone)]
pub enum RelationalCheckpointArtifactPolicy<'members> {
    Embedded,
    ExternalMembers {
        /// The destination's currently selected membership. Only these
        /// locators may be carried, regardless of another runtime's bindings.
        current_members: &'members [RelationalDurableMemberLocator],
        /// The destination's read port, retained after publication. Offline
        /// captures can omit it and cannot acknowledge a serving directory.
        source: Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    },
}

impl RelationalCheckpointArtifactPolicy<'_> {
    pub(crate) fn is_embedded(&self) -> bool {
        matches!(self, Self::Embedded)
    }
    pub(crate) fn current_members(
        &self,
    ) -> Option<std::collections::BTreeSet<RelationalDurableMemberLocator>> {
        match self {
            Self::Embedded => None,
            Self::ExternalMembers {
                current_members, ..
            } => Some(current_members.iter().copied().collect()),
        }
    }
    pub(crate) fn publication_source(
        &self,
    ) -> Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>> {
        match self {
            Self::Embedded => None,
            Self::ExternalMembers { source, .. } => source.clone(),
        }
    }
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
