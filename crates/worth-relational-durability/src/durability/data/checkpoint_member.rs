use std::sync::Arc;
use worth_relational_storage::internal_port as storage;

use storage::durability::medium::cold_source::RelationalDurableMemberLocator;

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
        source:
            Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>,
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
    ) -> Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>
    {
        match self {
            Self::Embedded => None,
            Self::ExternalMembers { source, .. } => source.clone(),
        }
    }
}

#[doc(hidden)]
pub trait RelationalCheckpointArtifactPolicyOwnerPort: Sized {
    fn is_embedded(&self) -> bool;

    fn publication_source(
        &self,
    ) -> Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>;
}

impl RelationalCheckpointArtifactPolicyOwnerPort for RelationalCheckpointArtifactPolicy<'_> {
    fn is_embedded(&self) -> bool {
        self.is_embedded()
    }

    fn publication_source(
        &self,
    ) -> Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>
    {
        self.publication_source()
    }
}
