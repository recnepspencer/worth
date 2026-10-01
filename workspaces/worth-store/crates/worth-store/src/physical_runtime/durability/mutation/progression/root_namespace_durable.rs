use worth_store_physical_format::RecordArtifactFile;

use super::root_prepared::RootPublicationPreparedCore;
use crate::physical_runtime::durability::PhysicalRootPublicationIdentity;
use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalRootPublicationMemberIdentity,
    RootPublicationPhysicalMutationMember, SettledPhysicalWork,
};

pub struct RootNamespaceDurablePhysicalMutationMembers {
    core: RootPublicationPreparedCore,
    replacement: SettledPhysicalWork,
    namespace_synchronization: SettledPhysicalWork,
}

impl RootNamespaceDurablePhysicalMutationMembers {
    pub(in crate::physical_runtime) fn retained_routing_metadata_bytes(&self) -> Option<u64> {
        self.core.candidate().retained_routing_metadata_bytes()
    }

    pub(in crate::physical_runtime) fn new(
        core: RootPublicationPreparedCore,
        replacement: SettledPhysicalWork,
        namespace_synchronization: SettledPhysicalWork,
    ) -> Self {
        Self {
            core,
            replacement,
            namespace_synchronization,
        }
    }

    pub const fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        self.core.group()
    }

    pub fn members(&self) -> &[PhysicalRootPublicationMemberIdentity] {
        self.core.members()
    }

    pub fn settled_members(&self) -> &[RootPublicationPhysicalMutationMember] {
        self.core.settled_members()
    }

    pub fn candidate_artifacts(&self) -> &[RecordArtifactFile] {
        self.core.candidate().artifacts()
    }

    pub fn source_root_generation(&self) -> u64 {
        self.core.candidate().source_root().generation()
    }

    pub fn current_root_generation(&self) -> u64 {
        self.core.candidate().successor_root().generation()
    }

    pub(in crate::physical_runtime) fn candidate_tier_epoch_anchor(&self) -> Option<[u8; 32]> {
        self.core.candidate().successor_root().tier_epoch_anchor()
    }

    pub fn namespace_effect_identity(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalEffectIdentity> {
        self.namespace_synchronization.effect_identity()
    }

    pub fn replacement_effect_identity(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalEffectIdentity> {
        self.replacement.effect_identity()
    }

    pub(in crate::physical_runtime) const fn identity(&self) -> PhysicalRootPublicationIdentity {
        self.core.identity()
    }

    pub(in crate::physical_runtime) const fn source_root(
        &self,
    ) -> &worth_store_physical_format::DurablePhysicalRootManifest {
        self.core.candidate().source_root()
    }

    pub(in crate::physical_runtime) fn transition_matches(&self) -> bool {
        self.core.transition_matches()
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        RootPublicationPreparedCore,
        SettledPhysicalWork,
        SettledPhysicalWork,
    ) {
        (self.core, self.replacement, self.namespace_synchronization)
    }
}
