use std::sync::Arc;

use worth_store_physical_format::{PersistedRecordIdentity, RecordFrameCoordinate};

use crate::physical_runtime::durability::settlement::{
    PhysicalMutationAcknowledgment, PhysicalMutationAcknowledgmentBasis,
    PhysicalMutationCompletedBreadth,
};
use crate::physical_runtime::{
    PhysicalDurabilityGroupMemberBinding, PhysicalDurabilityPolicyIdentity,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdentity,
    PhysicalMutationRequestFingerprint, RecordAppendObservation,
    RootPublicationPhysicalMutationMember,
};

pub struct CompletedPhysicalMutation {
    fact: Arc<CompletedPhysicalMutationFact>,
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::physical_runtime) struct CompletedPhysicalMutationFact {
    mutation: PhysicalMutationIdentity,
    idempotency: PhysicalMutationIdempotencyKeyIdentity,
    fingerprint: PhysicalMutationRequestFingerprint,
    group_binding: PhysicalDurabilityGroupMemberBinding,
    policy: PhysicalDurabilityPolicyIdentity,
    breadth: PhysicalMutationCompletedBreadth,
    records: Box<[PersistedRecordIdentity]>,
    completed_data_frames: Box<[RecordFrameCoordinate]>,
    observation: RecordAppendObservation,
    wal_frame_header_witness: Option<(u64, u64, [u8; 32], [u8; 32])>,
    release_head_effect: Option<worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1>,
}

impl CompletedPhysicalMutation {
    pub(in crate::physical_runtime) fn from_fact(
        fact: &Arc<CompletedPhysicalMutationFact>,
    ) -> Self {
        Self {
            fact: Arc::clone(fact),
        }
    }

    pub fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.fact.mutation
    }

    pub fn idempotency_identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.fact.idempotency
    }

    pub fn request_fingerprint(&self) -> PhysicalMutationRequestFingerprint {
        self.fact.fingerprint
    }

    pub fn completed_breadth(&self) -> PhysicalMutationCompletedBreadth {
        self.fact.breadth
    }

    pub fn persisted_records(&self) -> &[PersistedRecordIdentity] {
        &self.fact.records
    }

    pub(in crate::physical_runtime) fn completed_data_frame_coordinates(
        &self,
    ) -> &[RecordFrameCoordinate] {
        &self.fact.completed_data_frames
    }

    pub fn observation(&self) -> RecordAppendObservation {
        self.fact.observation
    }

    /// Exact checksum-admitted encoded C9 header from the settled selected
    /// member. Replayed terminal facts cannot manufacture this witness.
    pub(in crate::physical_runtime) fn wal_frame_header_witness(
        &self,
    ) -> Option<(u64, u64, [u8; 32], [u8; 32])> {
        self.fact.wal_frame_header_witness
    }

    pub(in crate::physical_runtime) fn selected_head_transition(
        &self,
    ) -> Option<&worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1> {
        self.fact.release_head_effect.as_ref()
    }

    pub fn into_acknowledgment(self) -> PhysicalMutationAcknowledgment {
        PhysicalMutationAcknowledgment::from_completed(PhysicalMutationAcknowledgmentBasis {
            mutation: self.fact.mutation,
            idempotency: self.fact.idempotency,
            fingerprint: self.fact.fingerprint,
            group_binding: self.fact.group_binding,
            policy: self.fact.policy,
            breadth: self.fact.breadth,
            records: self.fact.records.clone(),
            observation: self.fact.observation,
        })
    }
}

impl CompletedPhysicalMutationFact {
    /// The private C.8 pending claim and Store's fresh media rejoin prove that
    /// this exact durable descriptor is selected after redo. The original
    /// publication's performance counters were not retained by WAL, so this
    /// recovered fact carries no synthetic append observation.
    pub(in crate::physical_runtime) fn from_verified_pending_release(
        binding: &crate::physical_runtime::durability::PersistedPhysicalMutationAttemptBinding,
        descriptor: PersistedRecordIdentity,
        published_root_generation: u64,
    ) -> Arc<Self> {
        Self::from_persisted_terminal(
            binding,
            1,
            published_root_generation,
            vec![descriptor].into_boxed_slice(),
            RecordAppendObservation::unobserved_recovery(),
        )
    }

    pub(in crate::physical_runtime) fn from_root_member(
        member: &RootPublicationPhysicalMutationMember,
        fingerprint: PhysicalMutationRequestFingerprint,
        current_root_generation: u64,
    ) -> Arc<Self> {
        let wal_member = member.wal_member_basis();
        Arc::new(Self {
            mutation: member.mutation_identity(),
            idempotency: member.identity().idempotency_identity(),
            fingerprint,
            group_binding: member.identity().group_binding(),
            policy: member.wal_barrier_settlement().policy_identity(),
            breadth: PhysicalMutationCompletedBreadth::completed(
                member.persisted_records().len(),
                member.data_effect_count(),
                wal_member,
                current_root_generation,
            ),
            records: member.persisted_records().to_vec().into_boxed_slice(),
            completed_data_frames: member.completed_data_frame_coordinates(),
            observation: member.observation(),
            wal_frame_header_witness: member.encoded_frame_header_witness(),
            release_head_effect: member.selected_head_transition().cloned(),
        })
    }

    pub(in crate::physical_runtime) fn from_persisted_terminal(
        binding: &crate::physical_runtime::durability::PersistedPhysicalMutationAttemptBinding,
        data_effect_count: u32,
        current_root_generation: u64,
        records: Box<[PersistedRecordIdentity]>,
        observation: RecordAppendObservation,
    ) -> Arc<Self> {
        Arc::new(Self {
            mutation: binding.mutation(),
            idempotency: binding.idempotency_identity(),
            fingerprint: binding.fingerprint(),
            group_binding: binding.group(),
            policy: binding.policy_identity(),
            breadth: PhysicalMutationCompletedBreadth::completed(
                records.len(),
                data_effect_count as usize,
                binding.member(),
                current_root_generation,
            ),
            records,
            // Replayed terminal facts issue no new frame writes in this runtime.
            completed_data_frames: Box::new([]),
            observation,
            wal_frame_header_witness: None,
            release_head_effect: None,
        })
    }

    pub(in crate::physical_runtime) const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.mutation
    }

    pub(in crate::physical_runtime) const fn idempotency_identity(
        &self,
    ) -> PhysicalMutationIdempotencyKeyIdentity {
        self.idempotency
    }

    pub(in crate::physical_runtime) const fn request_fingerprint(
        &self,
    ) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }

    pub(in crate::physical_runtime) const fn breadth(&self) -> PhysicalMutationCompletedBreadth {
        self.breadth
    }

    pub(in crate::physical_runtime) const fn group_binding(
        &self,
    ) -> PhysicalDurabilityGroupMemberBinding {
        self.group_binding
    }

    pub(in crate::physical_runtime) fn persisted_records(&self) -> &[PersistedRecordIdentity] {
        &self.records
    }

    pub(in crate::physical_runtime) const fn observation(&self) -> RecordAppendObservation {
        self.observation
    }
}
