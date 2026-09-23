use crate::physical_runtime::{
    durability::CompletionBoundPhysicalWalBarrierSettlement, PhysicalDurabilityGroupMemberBinding,
    PhysicalMutationIdentity, PhysicalWalBarrierSettlement, WalAppendedPhysicalMutation,
    WalBarrierMember,
};

pub struct WalDurablePhysicalMutation {
    appended: WalAppendedPhysicalMutation,
    group_binding: PhysicalDurabilityGroupMemberBinding,
    settlement: PhysicalWalBarrierSettlement,
}

impl WalDurablePhysicalMutation {
    pub(in crate::physical_runtime) fn new(
        member: WalBarrierMember<WalAppendedPhysicalMutation>,
        settlement: CompletionBoundPhysicalWalBarrierSettlement,
    ) -> Self {
        let (group_binding, appended) = member.into_parts();
        Self {
            appended,
            group_binding,
            settlement: settlement.settlement(),
        }
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.appended.mutation_identity()
    }

    pub const fn member_basis(&self) -> crate::physical_runtime::PhysicalWalMemberBasis {
        self.appended.reserved().member_basis()
    }

    pub const fn group_binding(&self) -> PhysicalDurabilityGroupMemberBinding {
        self.group_binding
    }

    pub const fn barrier_settlement(&self) -> PhysicalWalBarrierSettlement {
        self.settlement
    }

    pub const fn appended(&self) -> &WalAppendedPhysicalMutation {
        &self.appended
    }

    pub(in crate::physical_runtime) fn data_frames(
        &self,
    ) -> &[crate::physical_runtime::durability::WalBoundPhysicalDataFrame] {
        self.appended.reserved().data().frames()
    }

    /// Whether this mutation's data plan carries record-preserving rewrite redo.
    pub(in crate::physical_runtime) const fn carries_rewrite(&self) -> bool {
        self.appended.reserved().data().rewrite().is_some()
    }

    pub(in crate::physical_runtime) const fn root_projection(
        &self,
    ) -> &crate::physical_runtime::PreparedPhysicalRootProjection {
        self.appended.reserved().root_projection()
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        WalAppendedPhysicalMutation,
        PhysicalDurabilityGroupMemberBinding,
        PhysicalWalBarrierSettlement,
    ) {
        (self.appended, self.group_binding, self.settlement)
    }
}
