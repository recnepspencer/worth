use crate::physical_runtime::{
    durability::CompletionBoundPhysicalWalBarrierSettlement, PhysicalDurabilityGroupMemberBinding,
    PhysicalMutationIdentity, PhysicalWalBarrierSettlement, WalAppendedPhysicalMutation,
    WalBarrierMember,
};

pub struct WalDurablePhysicalMutation {
    appended: WalAppendedPhysicalMutation,
    group_binding: PhysicalDurabilityGroupMemberBinding,
    settlement: PhysicalWalBarrierSettlement,
    completed_data_prefix: Vec<crate::physical_runtime::PhysicalDataEffectSettlement>,
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
            completed_data_prefix: Vec::new(),
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
    ) -> Option<&[crate::physical_runtime::durability::WalBoundPhysicalDataFrame]> {
        self.appended.reserved().data().frames()
    }

    pub(in crate::physical_runtime) fn source_copy(
        &self,
    ) -> Option<(
        &crate::physical_runtime::record_serving::AdoptedExtentCopy,
        worth_store_wal::WalLsnRange,
    )> {
        self.appended.reserved().data().source_copy()
    }

    pub(in crate::physical_runtime) fn completed_data_frames(&self) -> usize {
        self.completed_data_prefix.len()
    }

    pub(in crate::physical_runtime) fn take_completed_data_prefix(
        &mut self,
    ) -> Vec<crate::physical_runtime::PhysicalDataEffectSettlement> {
        std::mem::take(&mut self.completed_data_prefix)
    }

    pub(in crate::physical_runtime) fn retain_completed_data_prefix(
        &mut self,
        prefix: Vec<crate::physical_runtime::PhysicalDataEffectSettlement>,
    ) {
        assert!(self.completed_data_prefix.is_empty());
        self.completed_data_prefix = prefix;
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
