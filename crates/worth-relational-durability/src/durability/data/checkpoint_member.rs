use std::sync::Arc;
use worth_relational_storage::internal_port as storage;

use storage::durability::medium::cold_source::RelationalDurableMemberLocator;

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
        source:
            Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>,
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
    ) -> Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>
    {
        let Self::ExternalMembers { source, .. } = self;
        source.clone()
    }
}

#[doc(hidden)]
pub trait RelationalCheckpointArtifactPolicyOwnerPort: Sized {
    fn publication_source(
        &self,
    ) -> Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>;
}

impl RelationalCheckpointArtifactPolicyOwnerPort for RelationalCheckpointArtifactPolicy<'_> {
    fn publication_source(
        &self,
    ) -> Option<Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>>
    {
        self.publication_source()
    }
}
