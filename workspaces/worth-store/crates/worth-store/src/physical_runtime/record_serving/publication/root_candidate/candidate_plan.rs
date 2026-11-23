use std::num::NonZeroU64;

use worth_proof::NonEmpty;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, RecordArtifactFile,
};

use super::super::PublicationPlan;
use super::{
    candidate::PreparedPhysicalRootCandidate, planning_members::RootPublicationPlanningCore,
};
use crate::physical_runtime::durability::{
    PhysicalRootPublicationIdentity, PhysicalRootPublicationTransition,
};
use crate::physical_runtime::record_serving::{
    residency::frame_ports::CandidateFrameSet, RecordAppendDenial,
};
use crate::physical_runtime::{
    PhysicalDurabilityGroupBasis, PhysicalRootPublicationMemberIdentity,
    RootPublicationPhysicalMutationMember,
};

pub struct RootPublicationCandidatePlan {
    basis: Vec<RootPublicationCandidateBasis>,
    plan: Vec<PublicationPlan>,
}

pub(in crate::physical_runtime::record_serving) struct RootPublicationCandidateBasis {
    core: RootPublicationPlanningCore,
    source_root: DurablePhysicalRootManifest,
    successor_free_space: DurableFreeSpaceManifestHeader,
    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    allocation_bytes: NonZeroU64,
    transition: PhysicalRootPublicationTransition,
}

pub(in crate::physical_runtime) struct WrittenRootPublicationCandidate {
    core: RootPublicationPlanningCore,
    candidate: Box<PreparedPhysicalRootCandidate>,
    transition: PhysicalRootPublicationTransition,
}

impl RootPublicationCandidatePlan {
    pub(in crate::physical_runtime::record_serving) fn new(
        core: RootPublicationPlanningCore,
        source_root: DurablePhysicalRootManifest,
        successor_free_space: DurableFreeSpaceManifestHeader,
        allocation_bytes: NonZeroU64,
        transition: PhysicalRootPublicationTransition,
        plan: PublicationPlan,
    ) -> Self {
        Self {
            basis: vec![RootPublicationCandidateBasis {
                core,
                source_root,
                successor_free_space,
                allocation_bytes,
                transition,
            }],
            plan: vec![plan],
        }
    }

    pub(in crate::physical_runtime) const fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        self.basis.as_slice()[0].core.group_basis()
    }

    pub(in crate::physical_runtime) fn member_identities(
        &self,
    ) -> &[PhysicalRootPublicationMemberIdentity] {
        self.basis[0].core.member_identities()
    }

    pub(in crate::physical_runtime) fn settled_members(
        &self,
    ) -> &[RootPublicationPhysicalMutationMember] {
        self.basis[0].core.settled_members()
    }

    pub(in crate::physical_runtime) const fn candidate_generation(&self) -> u64 {
        self.plan.as_slice()[0].generation
    }

    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(in crate::physical_runtime::record_serving) const fn allocation_bytes(&self) -> NonZeroU64 {
        self.basis.as_slice()[0].allocation_bytes
    }

    pub(in crate::physical_runtime::record_serving) fn frame_set(
        &self,
    ) -> Result<CandidateFrameSet, RecordAppendDenial> {
        self.plan[0].root_candidate_frame_set()
    }

    pub(in crate::physical_runtime::record_serving) fn into_write_parts(
        mut self,
    ) -> (RootPublicationCandidateBasis, PublicationPlan) {
        (
            self.basis.pop().expect("candidate plan holds one basis"),
            self.plan.pop().expect("candidate plan holds one plan"),
        )
    }
}

impl RootPublicationCandidateBasis {
    pub(in crate::physical_runtime::record_serving) fn mark_effect_started(&mut self) {
        self.transition.mark_effect_started();
    }

    pub(in crate::physical_runtime::record_serving) fn require_inspection(&mut self) {
        self.transition.require_inspection();
    }

    pub(in crate::physical_runtime::record_serving) fn restore_proven_no_effect(
        mut self,
        plan: PublicationPlan,
    ) -> RootPublicationCandidatePlan {
        self.transition.prove_no_effect();
        RootPublicationCandidatePlan {
            basis: vec![self],
            plan: vec![plan],
        }
    }

    pub(in crate::physical_runtime::record_serving) fn restore_inspection_required(
        self,
        plan: PublicationPlan,
    ) -> RootPublicationCandidatePlan {
        RootPublicationCandidatePlan {
            basis: vec![self],
            plan: vec![plan],
        }
    }

    pub(in crate::physical_runtime::record_serving) fn complete_write(
        self,
        plan: PublicationPlan,
        artifacts: Box<[RecordArtifactFile]>,
    ) -> WrittenRootPublicationCandidate {
        WrittenRootPublicationCandidate {
            core: self.core,
            candidate: Box::new(PreparedPhysicalRootCandidate::new(
                self.source_root,
                self.successor_free_space,
                plan,
                artifacts,
            )),
            transition: self.transition,
        }
    }
}

impl WrittenRootPublicationCandidate {
    pub(in crate::physical_runtime) const fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        self.core.group_basis()
    }

    pub(in crate::physical_runtime) fn member_identities(
        &self,
    ) -> &[PhysicalRootPublicationMemberIdentity] {
        self.core.member_identities()
    }

    pub(in crate::physical_runtime) fn settled_members(
        &self,
    ) -> &[RootPublicationPhysicalMutationMember] {
        self.core.settled_members()
    }

    pub(in crate::physical_runtime) fn candidate(&self) -> &PreparedPhysicalRootCandidate {
        &self.candidate
    }

    pub(in crate::physical_runtime::record_serving) const fn identity(
        &self,
    ) -> PhysicalRootPublicationIdentity {
        self.transition.identity()
    }

    pub(in crate::physical_runtime::record_serving) fn require_inspection(&mut self) {
        self.transition.require_inspection();
    }

    pub(in crate::physical_runtime::record_serving) fn into_parts(
        self,
    ) -> (
        PhysicalDurabilityGroupBasis,
        NonEmpty<RootPublicationPhysicalMutationMember>,
        PreparedPhysicalRootCandidate,
        PhysicalRootPublicationTransition,
    ) {
        (
            self.core.group_basis(),
            self.core.into_settled_members(),
            *self.candidate,
            self.transition,
        )
    }
}
