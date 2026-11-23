use worth_store_physical_format::RecordArtifactFile;

use super::root_prepared::RootPublicationPreparedCore;
use crate::physical_runtime::durability::PhysicalRootPublicationIdentity;
use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalRootPublicationMemberIdentity,
    RootPublicationPhysicalMutationMember, SettledPhysicalWork,
};

pub struct RootReplacedPhysicalMutationMembers {
    core: RootPublicationPreparedCore,
    replacement: Box<SettledPhysicalWork>,
}

impl RootReplacedPhysicalMutationMembers {
    pub(in crate::physical_runtime) fn new(
        core: RootPublicationPreparedCore,
        replacement: SettledPhysicalWork,
    ) -> Self {
        Self {
            core,
            replacement: Box::new(replacement),
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

    pub fn candidate_root_generation(&self) -> u64 {
        self.core.candidate().successor_root().generation()
    }

    pub fn replacement_effect_identity(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalEffectIdentity> {
        self.replacement.effect_identity()
    }

    pub(in crate::physical_runtime) const fn identity(&self) -> PhysicalRootPublicationIdentity {
        self.core.identity()
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (RootPublicationPreparedCore, Box<SettledPhysicalWork>) {
        (self.core, self.replacement)
    }
}
