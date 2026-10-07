use super::DataDispatchedPhysicalMutation;
use crate::physical_runtime::{
    durability::{
        CompletionBoundPhysicalDataSettlement, PhysicalDataEffectSettlement,
        WalRangeReservedPhysicalMutationBasis,
    },
    PhysicalDurabilityGroupMemberBinding, PhysicalMutationIdentity, PhysicalWalAppendSettlement,
    PhysicalWalBarrierSettlement, PreparedPhysicalRootProjection,
};

pub struct DataSettledPhysicalMutation {
    dispatched: DataDispatchedPhysicalMutation,
}

pub(in crate::physical_runtime) struct SettledPhysicalMutationBasis {
    reserved: WalRangeReservedPhysicalMutationBasis,
    wal_append: PhysicalWalAppendSettlement,
    group_binding: PhysicalDurabilityGroupMemberBinding,
    wal_barrier: PhysicalWalBarrierSettlement,
    data_effects: Vec<PhysicalDataEffectSettlement>,
}

impl DataSettledPhysicalMutation {
    pub(in crate::physical_runtime) fn new(
        settlement: CompletionBoundPhysicalDataSettlement,
    ) -> Self {
        Self {
            dispatched: settlement.into_dispatched(),
        }
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.dispatched.mutation_identity()
    }

    pub const fn dispatched(&self) -> &DataDispatchedPhysicalMutation {
        &self.dispatched
    }

    pub fn prepared_root_source_generation(&self) -> u64 {
        self.dispatched
            .durable()
            .root_projection()
            .source_root_generation()
    }

    pub const fn group_binding(
        &self,
    ) -> crate::physical_runtime::PhysicalDurabilityGroupMemberBinding {
        self.dispatched.durable().group_binding()
    }

    pub const fn wal_member_identity(&self) -> crate::physical_runtime::PhysicalWalMemberIdentity {
        self.dispatched.durable().member_basis().member_identity()
    }

    pub const fn idempotency_identity(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationIdempotencyKeyIdentity {
        self.dispatched
            .durable()
            .appended()
            .reserved()
            .idempotency_identity()
    }

    pub(in crate::physical_runtime) fn into_root_publication_parts(
        self,
    ) -> (SettledPhysicalMutationBasis, PreparedPhysicalRootProjection) {
        let (durable, data_effects) = self.dispatched.into_parts();
        let (appended, group_binding, wal_barrier) = durable.into_parts();
        let (reserved, wal_append) = appended.into_parts();
        let (reserved, root) = reserved.into_root_publication_parts();
        (
            SettledPhysicalMutationBasis {
                reserved,
                wal_append,
                group_binding,
                wal_barrier,
                data_effects,
            },
            root,
        )
    }
}

impl SettledPhysicalMutationBasis {
    pub(in crate::physical_runtime) fn encoded_frame_header_witness(
        &self,
    ) -> Option<(u64, u64, [u8; 32], [u8; 32])> {
        self.reserved.encoded_frame_header_witness()
    }

    pub(in crate::physical_runtime) fn source_copy_evidence(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalExtentCopySettlementObservation> {
        self.reserved.source_copy_evidence()
    }
    pub(in crate::physical_runtime) const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.reserved.mutation_identity()
    }

    pub(in crate::physical_runtime) const fn idempotency_identity(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationIdempotencyKeyIdentity {
        self.reserved.idempotency_identity()
    }

    pub(in crate::physical_runtime) const fn group_binding(
        &self,
    ) -> PhysicalDurabilityGroupMemberBinding {
        self.group_binding
    }

    pub(in crate::physical_runtime) const fn wal_barrier(&self) -> PhysicalWalBarrierSettlement {
        self.wal_barrier
    }

    pub(in crate::physical_runtime) const fn wal_append(&self) -> &PhysicalWalAppendSettlement {
        &self.wal_append
    }

    pub(in crate::physical_runtime) const fn wal_member_basis(
        &self,
    ) -> crate::physical_runtime::PhysicalWalMemberBasis {
        self.reserved.member_basis()
    }

    pub(in crate::physical_runtime) fn data_effects(&self) -> &[PhysicalDataEffectSettlement] {
        &self.data_effects
    }
}
