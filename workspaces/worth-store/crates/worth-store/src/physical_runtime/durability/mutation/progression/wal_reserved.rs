use worth_store_physical_backend::ArtifactTreeFile;
use worth_store_wal::{PlannedWalFrameAppend, WalAppendFrontier};

use crate::physical_runtime::{
    durability::{
        AdmittedPhysicalMutation, AllocatedPhysicalMutationAttemptBinding, WalBoundPhysicalDataPlan,
    },
    AdmittedRecordPlacementPolicy, CanonicalRedoRecords, PhysicalGroupQueueAdmissionTick,
    PhysicalMutationDeadline, PhysicalMutationIdentity, PhysicalMutationResourceShape,
    PhysicalSignalProfileIdentity, PhysicalWalAppendDeclaration, PhysicalWalMemberBasis,
    PhysicalWorkSemanticBasis, PlannedPhysicalMutationParts, PreparedPhysicalMutation,
    PreparedPhysicalMutationContext, PreparedPhysicalRootProjection, RecordAppendBatch,
};

pub struct WalRangeReservedPhysicalMutation {
    basis: WalRangeReservedPhysicalMutationBasis,
    root: PreparedPhysicalRootProjection,
}

pub(in crate::physical_runtime) struct WalRangeReservedPhysicalMutationBasis {
    binding: AllocatedPhysicalMutationAttemptBinding,
    member: PhysicalWalMemberBasis,
    redo: CanonicalRedoRecords,
    data: WalBoundPhysicalDataPlan,
    frame: PlannedWalFrameAppend,
    artifact: ArtifactTreeFile,
    declaration: PhysicalWalAppendDeclaration,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    group_queue_admission: PhysicalGroupQueueAdmissionTick,
    signal_profile: PhysicalSignalProfileIdentity,
    durability_policy_basis: PhysicalWorkSemanticBasis,
    resources: PhysicalMutationResourceShape,
    start: crate::physical_runtime::PhysicalMutationStartPort,
}

impl WalRangeReservedPhysicalMutation {
    pub(in crate::physical_runtime) fn new(
        binding: AllocatedPhysicalMutationAttemptBinding,
        member: PhysicalWalMemberBasis,
        redo: CanonicalRedoRecords,
        data: WalBoundPhysicalDataPlan,
        root: PreparedPhysicalRootProjection,
        frame: PlannedWalFrameAppend,
        artifact: ArtifactTreeFile,
        declaration: PhysicalWalAppendDeclaration,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        group_queue_admission: PhysicalGroupQueueAdmissionTick,
        signal_profile: PhysicalSignalProfileIdentity,
        durability_policy_basis: PhysicalWorkSemanticBasis,
        resources: PhysicalMutationResourceShape,
        start: crate::physical_runtime::PhysicalMutationStartPort,
    ) -> Self {
        Self {
            basis: WalRangeReservedPhysicalMutationBasis {
                binding,
                member,
                redo,
                data,
                frame,
                artifact,
                declaration,
                placement,
                deadline,
                group_queue_admission,
                signal_profile,
                durability_policy_basis,
                resources,
                start,
            },
            root,
        }
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.basis.mutation_identity()
    }

    pub const fn request_fingerprint(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationRequestFingerprint {
        self.basis.binding.fingerprint()
    }

    pub const fn bound_redo_digest(&self) -> [u8; 32] {
        self.basis.binding.redo_digest()
    }

    pub(in crate::physical_runtime) fn persisted_attempt_binding(
        &self,
    ) -> crate::physical_runtime::durability::PersistedPhysicalMutationAttemptBinding {
        crate::physical_runtime::durability::PersistedPhysicalMutationAttemptBinding::from_allocated(
            &self.basis.binding,
        )
    }

    pub const fn member_basis(&self) -> PhysicalWalMemberBasis {
        self.basis.member
    }

    pub const fn group_binding(
        &self,
    ) -> crate::physical_runtime::PhysicalDurabilityGroupMemberBinding {
        self.basis.binding.group_binding()
    }

    pub const fn idempotency_identity(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationIdempotencyKeyIdentity {
        self.basis.binding.key().identity()
    }

    pub fn redo(&self) -> &CanonicalRedoRecords {
        &self.basis.redo
    }

    pub(in crate::physical_runtime) const fn data(&self) -> &WalBoundPhysicalDataPlan {
        &self.basis.data
    }

    pub(in crate::physical_runtime) const fn root_projection(
        &self,
    ) -> &PreparedPhysicalRootProjection {
        &self.root
    }

    pub fn artifact(&self) -> &ArtifactTreeFile {
        &self.basis.artifact
    }

    pub const fn declaration(&self) -> PhysicalWalAppendDeclaration {
        self.basis.declaration
    }

    pub const fn placement(&self) -> AdmittedRecordPlacementPolicy {
        self.basis.placement
    }

    pub const fn deadline(&self) -> PhysicalMutationDeadline {
        self.basis.deadline
    }

    pub const fn group_queue_admission_tick(&self) -> PhysicalGroupQueueAdmissionTick {
        self.basis.group_queue_admission
    }

    pub const fn signal_profile(&self) -> PhysicalSignalProfileIdentity {
        self.basis.signal_profile
    }

    pub const fn resources(&self) -> PhysicalMutationResourceShape {
        self.basis.resources
    }

    pub fn durability_policy_basis(&self) -> PhysicalWorkSemanticBasis {
        self.basis.durability_policy_basis.clone()
    }

    pub fn encoded_frame(&self) -> &[u8] {
        self.basis.frame.frame().encoded_frame()
    }

    pub const fn resulting_frontier(&self) -> WalAppendFrontier {
        self.basis.frame.resulting_frontier()
    }

    pub(in crate::physical_runtime) fn into_prepared_after_no_effect(
        self,
    ) -> PreparedPhysicalMutation {
        let manifest_capacity_transition = self.root.manifest_capacity_transition();
        let binding = self.basis.binding.release_wal_allocation();
        let batch = RecordAppendBatch::from_prepared_record_bytes(
            self.basis.redo.into_prepared_record_bytes(),
        );
        PreparedPhysicalMutation::from_planned_parts(PlannedPhysicalMutationParts {
            admission: AdmittedPhysicalMutation::Fresh(binding),
            batch,
            data: self.basis.data.into_prepared(),
            root: self.root,
            context: PreparedPhysicalMutationContext {
                placement: self.basis.placement,
                manifest_capacity_transition,
                deadline: self.basis.deadline,
                group_queue_admission: self.basis.group_queue_admission,
                signal_profile: self.basis.signal_profile,
                durability_policy_basis: self.basis.durability_policy_basis,
                resources: self.basis.resources,
                start: self.basis.start,
                selected_segment_rewrite: false,
                rewrite_pages: 0,
                source_root_generation: 0,
                rewrite_anchor: None,
            },
        })
    }

    pub(in crate::physical_runtime) fn into_root_publication_parts(
        self,
    ) -> (
        WalRangeReservedPhysicalMutationBasis,
        PreparedPhysicalRootProjection,
    ) {
        (self.basis, self.root)
    }
}

impl WalRangeReservedPhysicalMutationBasis {
    pub(in crate::physical_runtime) const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.member.mutation_identity()
    }

    pub(in crate::physical_runtime) const fn idempotency_identity(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationIdempotencyKeyIdentity {
        self.binding.key().identity()
    }

    pub(in crate::physical_runtime) const fn member_basis(&self) -> PhysicalWalMemberBasis {
        self.member
    }
}
