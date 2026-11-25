use std::sync::Arc;

use crate::durability::medium::RelationalDurableMemberLocator;

/// Placement of the member set of one native checkpoint. All native checkpoints
/// store members externally in cold storage; embedded placement is retired.
#[derive(Debug, Clone)]
pub enum RelationalCheckpointArtifactPolicy<'members> {
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
    pub(crate) fn current_members(
        &self,
    ) -> Option<std::collections::BTreeSet<RelationalDurableMemberLocator>> {
        let Self::ExternalMembers {
            current_members, ..
        } = self;
        Some(current_members.iter().copied().collect())
    }
    pub(crate) fn publication_source(
        &self,
    ) -> Option<Arc<dyn crate::durability::medium::RelationalDurableColdSource>> {
        let Self::ExternalMembers { source, .. } = self;
        source.clone()
    }
}

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
