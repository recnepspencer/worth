use std::num::NonZeroU64;

use worth_proof::NonEmpty;

use crate::physical_runtime::record_serving::planning::{
    merge_settled_root_projections, prepared_payload::PreparedRecordPayloadPlan,
    RejectedSettledRootProjections,
};
use crate::physical_runtime::{
    DataSettledPhysicalMutationMembers, PhysicalDurabilityGroupBasis,
    PhysicalRootPublicationMemberIdentity, PreparedPhysicalRootProjection,
    RootPublicationPhysicalMutationMember,
};

/// Exact settled membership and candidate material retained across pre-effect
/// root-planning denials.
///
/// Callers can inspect identity and proof-bearing members but cannot construct,
/// alter, reorder, or extract candidate projections.
pub struct RootPublicationPlanningMembers {
    core: RootPublicationPlanningCore,
    projection: RootPublicationPlanningProjection,
}

pub(in crate::physical_runtime::record_serving) struct RootPublicationPlanningCore {
    group: PhysicalDurabilityGroupBasis,
    member_identities: Box<[PhysicalRootPublicationMemberIdentity]>,
    members: NonEmpty<RootPublicationPhysicalMutationMember>,
}

enum RootPublicationPlanningProjection {
    Unmerged(NonEmpty<PreparedPhysicalRootProjection>),
    Merged {
        prepared: PreparedRecordPayloadPlan,
        allocation_bytes: NonZeroU64,
    },
}

impl RootPublicationPlanningMembers {
    pub(in crate::physical_runtime::record_serving) fn from_settled(
        settled: DataSettledPhysicalMutationMembers,
    ) -> Self {
        let (group, settled_members) = settled.into_parts();
        let mut members = Vec::with_capacity(settled_members.len());
        let mut projections = Vec::with_capacity(settled_members.len());
        for settled_member in settled_members.into_vec() {
            let (settled_basis, mut projection) = settled_member.into_root_publication_parts();
            let transfer_count = settled_basis.source_copy_evidence().map_or_else(
                || u64::try_from(settled_basis.data_effects().len()).unwrap_or(u64::MAX),
                |copy| {
                    u64::from(
                        copy.frame_writes()
                            .checked_sub(1)
                            .expect("validated source copy writes include its manifest"),
                    )
                },
            );
            projection.settle_data_observation(transfer_count);
            let completion = projection.completion_projection();
            members.push(RootPublicationPhysicalMutationMember::new(
                settled_basis,
                completion,
                projection.recovery_release_head_effect().cloned(),
            ));
            projections.push(projection);
        }
        let members = require_nonempty(members);
        let member_identities = members
            .as_slice()
            .iter()
            .map(RootPublicationPhysicalMutationMember::identity)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            core: RootPublicationPlanningCore {
                group,
                member_identities,
                members,
            },
            projection: RootPublicationPlanningProjection::Unmerged(require_nonempty(projections)),
        }
    }

    pub const fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        self.core.group
    }

    pub fn member_identities(&self) -> &[PhysicalRootPublicationMemberIdentity] {
        &self.core.member_identities
    }

    pub fn settled_members(&self) -> &[RootPublicationPhysicalMutationMember] {
        self.core.members.as_slice()
    }

    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(in crate::physical_runtime) fn source_root_generation(&self) -> u64 {
        match &self.projection {
            RootPublicationPlanningProjection::Unmerged(projections) => {
                projections.first().source_root_generation()
            }
            RootPublicationPlanningProjection::Merged { prepared, .. } => {
                prepared.source_root.generation()
            }
        }
    }

    pub(in crate::physical_runtime::record_serving) fn merge_candidate_projections(
        self,
    ) -> Result<Self, (Self, RejectedSettledRootProjections)> {
        let Self { core, projection } = self;
        let RootPublicationPlanningProjection::Unmerged(projections) = projection else {
            return Ok(Self { core, projection });
        };
        match merge_settled_root_projections(projections) {
            Ok(merged) => {
                let (prepared, allocation_bytes) = merged.into_parts();
                Ok(Self {
                    core,
                    projection: RootPublicationPlanningProjection::Merged {
                        prepared,
                        allocation_bytes,
                    },
                })
            }
            Err((projections, rejected)) => Err((
                Self {
                    core,
                    projection: RootPublicationPlanningProjection::Unmerged(projections),
                },
                rejected,
            )),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn into_merged_parts(
        self,
    ) -> (
        RootPublicationPlanningCore,
        PreparedRecordPayloadPlan,
        NonZeroU64,
    ) {
        let RootPublicationPlanningProjection::Merged {
            prepared,
            allocation_bytes,
        } = self.projection
        else {
            unreachable!("root candidate construction requires merged planning")
        };
        (self.core, prepared, allocation_bytes)
    }

    pub(in crate::physical_runtime::record_serving) fn allocation_bytes(&self) -> NonZeroU64 {
        match &self.projection {
            RootPublicationPlanningProjection::Merged {
                allocation_bytes, ..
            } => *allocation_bytes,
            RootPublicationPlanningProjection::Unmerged(_) => {
                unreachable!("root candidate allocation requires merged planning")
            }
        }
    }

    pub(in crate::physical_runtime::record_serving) fn from_merged_parts(
        core: RootPublicationPlanningCore,
        prepared: PreparedRecordPayloadPlan,
        allocation_bytes: NonZeroU64,
    ) -> Self {
        Self {
            core,
            projection: RootPublicationPlanningProjection::Merged {
                prepared,
                allocation_bytes,
            },
        }
    }
}

impl RootPublicationPlanningCore {
    pub(super) const fn group_basis(&self) -> PhysicalDurabilityGroupBasis {
        self.group
    }

    pub(super) fn member_identities(&self) -> &[PhysicalRootPublicationMemberIdentity] {
        &self.member_identities
    }

    pub(super) fn settled_members(&self) -> &[RootPublicationPhysicalMutationMember] {
        self.members.as_slice()
    }

    pub(super) fn into_settled_members(self) -> NonEmpty<RootPublicationPhysicalMutationMember> {
        self.members
    }
}

fn require_nonempty<T>(values: Vec<T>) -> NonEmpty<T> {
    NonEmpty::try_from_vec(values)
        .unwrap_or_else(|_| unreachable!("settled root membership is nonempty"))
}
