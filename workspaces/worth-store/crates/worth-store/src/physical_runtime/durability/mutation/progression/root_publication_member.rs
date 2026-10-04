use worth_store_physical_format::{PersistedRecordIdentity, RecordFrameCoordinate};

use crate::physical_runtime::durability::SettledPhysicalMutationBasis;
use crate::physical_runtime::{
    PhysicalMutationIdentity, PhysicalRootPublicationMemberIdentity,
    PreparedRecordCompletionProjection, RecordAppendObservation,
};

/// The exact settled mutation carried through one shared root publication.
///
/// Construction is private to the data-settlement/root-projection join. Root
/// phase transitions move this value and cannot replace it with identity-only
/// bookkeeping.
pub struct RootPublicationPhysicalMutationMember {
    identity: PhysicalRootPublicationMemberIdentity,
    settled: SettledPhysicalMutationBasis,
    completion: PreparedRecordCompletionProjection,
    release_head_effect: Option<worth_store_physical_format::PersistedReleaseHeadClaim>,
}

impl RootPublicationPhysicalMutationMember {
    pub(in crate::physical_runtime) fn encoded_frame_header_witness(
        &self,
    ) -> Option<(u64, u64, [u8; 32], [u8; 32])> {
        self.settled.encoded_frame_header_witness()
    }

    pub(in crate::physical_runtime) fn new(
        settled: SettledPhysicalMutationBasis,
        completion: PreparedRecordCompletionProjection,
        release_head_effect: Option<worth_store_physical_format::PersistedReleaseHeadClaim>,
    ) -> Self {
        let binding = settled.group_binding();
        let identity = PhysicalRootPublicationMemberIdentity::new(
            settled.mutation_identity(),
            binding.member_identity(),
            settled.idempotency_identity(),
            binding,
        );
        Self {
            identity,
            settled,
            completion,
            release_head_effect,
        }
    }

    pub(in crate::physical_runtime) fn selected_head_claim(
        &self,
    ) -> Option<&worth_store_physical_format::PersistedReleaseHeadClaim> {
        self.release_head_effect.as_ref()
    }

    pub const fn identity(&self) -> PhysicalRootPublicationMemberIdentity {
        self.identity
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.identity.mutation_identity()
    }

    pub fn persisted_records(&self) -> &[PersistedRecordIdentity] {
        self.completion.records()
    }

    pub(in crate::physical_runtime) fn completed_data_frame_coordinates(
        &self,
    ) -> Box<[RecordFrameCoordinate]> {
        self.settled
            .data_effects()
            .iter()
            .map(|effect| effect.coordinate())
            .collect()
    }

    /// Projects one completed physical record into the serving read identity.
    ///
    /// The projection carries no mutation, acknowledgment, or root authority.
    pub fn record_id(&self, index: usize) -> Option<crate::physical_runtime::PhysicalRecordId> {
        self.completion
            .records()
            .get(index)
            .copied()
            .map(crate::physical_runtime::PhysicalRecordId::from_persisted)
    }

    pub const fn observation(&self) -> RecordAppendObservation {
        self.completion.observation()
    }

    pub fn data_effect_count(&self) -> usize {
        self.source_copy_evidence().map_or_else(
            || self.settled.data_effects().len(),
            |copy| copy.frame_writes() as usize,
        )
    }

    pub fn source_copy_evidence(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalExtentCopySettlementObservation> {
        self.settled.source_copy_evidence()
    }

    pub const fn wal_append_settlement(
        &self,
    ) -> &crate::physical_runtime::PhysicalWalAppendSettlement {
        self.settled.wal_append()
    }

    pub const fn wal_barrier_settlement(
        &self,
    ) -> crate::physical_runtime::PhysicalWalBarrierSettlement {
        self.settled.wal_barrier()
    }

    pub const fn wal_member_basis(&self) -> crate::physical_runtime::PhysicalWalMemberBasis {
        self.settled.wal_member_basis()
    }
}
