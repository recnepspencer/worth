use crate::physical_runtime::{
    durability::{AdmittedPhysicalMutation, PreparedPhysicalDataPlan},
    AdmittedRecordPlacementPolicy, PhysicalGroupQueueAdmissionTick, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdempotencyLease,
    PhysicalMutationIdentity, PhysicalMutationRequestFingerprint, PhysicalSignalProfileIdentity,
    PhysicalWorkSemanticBasis, PreparedPhysicalRootProjection, RecordAppendBatch,
};

use super::CanonicalPayloadMaterializationObservation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationAdmissionDisposition {
    Fresh,
    DuplicateUnresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalMutationResourceShape {
    record_count: u32,
    payload_bytes: u64,
    prepared_payload_bytes: u64,
}

pub struct PreparedPhysicalMutation {
    admission: AdmittedPhysicalMutation,
    data: PreparedPhysicalMutationData,
    placement: AdmittedRecordPlacementPolicy,
    manifest_capacity_transition: super::PhysicalManifestCapacityTransition,
    deadline: PhysicalMutationDeadline,
    group_queue_admission: PhysicalGroupQueueAdmissionTick,
    signal_profile: PhysicalSignalProfileIdentity,
    durability_policy_basis: PhysicalWorkSemanticBasis,
    resources: PhysicalMutationResourceShape,
    start: crate::physical_runtime::PhysicalMutationStartPort,
    selected_segment_rewrite: bool,
    source_root_generation: u64,
}

pub(in crate::physical_runtime) struct PreparedPhysicalMutationContext {
    pub(in crate::physical_runtime) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime) manifest_capacity_transition:
        super::PhysicalManifestCapacityTransition,
    pub(in crate::physical_runtime) deadline: PhysicalMutationDeadline,
    pub(in crate::physical_runtime) group_queue_admission: PhysicalGroupQueueAdmissionTick,
    pub(in crate::physical_runtime) signal_profile: PhysicalSignalProfileIdentity,
    pub(in crate::physical_runtime) durability_policy_basis: PhysicalWorkSemanticBasis,
    pub(in crate::physical_runtime) resources: PhysicalMutationResourceShape,
    pub(in crate::physical_runtime) start: crate::physical_runtime::PhysicalMutationStartPort,
}

pub(in crate::physical_runtime) struct PlannedPhysicalMutationParts {
    pub(in crate::physical_runtime) admission: AdmittedPhysicalMutation,
    pub(in crate::physical_runtime) batch: RecordAppendBatch,
    pub(in crate::physical_runtime) data: PreparedPhysicalDataPlan,
    pub(in crate::physical_runtime) root: PreparedPhysicalRootProjection,
    pub(in crate::physical_runtime) context: PreparedPhysicalMutationContext,
}

enum PreparedPhysicalMutationData {
    Unplanned {
        batch: RecordAppendBatch,
        materialization: CanonicalPayloadMaterializationObservation,
    },
    Planned {
        batch: RecordAppendBatch,
        data: PreparedPhysicalDataPlan,
        root: PreparedPhysicalRootProjection,
    },
}

impl PhysicalMutationResourceShape {
    pub(in crate::physical_runtime::record_serving) const fn prepared(
        record_count: u32,
        payload_bytes: u64,
    ) -> Self {
        Self {
            record_count,
            payload_bytes,
            prepared_payload_bytes: payload_bytes,
        }
    }

    pub const fn record_count(self) -> u32 {
        self.record_count
    }

    pub const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }

    pub const fn prepared_payload_bytes(self) -> u64 {
        self.prepared_payload_bytes
    }
}

impl PreparedPhysicalMutation {
    pub(in crate::physical_runtime::record_serving) fn new(
        admission: AdmittedPhysicalMutation,
        batch: RecordAppendBatch,
        materialization: CanonicalPayloadMaterializationObservation,
        context: PreparedPhysicalMutationContext,
    ) -> Self {
        Self {
            admission,
            data: PreparedPhysicalMutationData::Unplanned {
                batch,
                materialization,
            },
            placement: context.placement,
            manifest_capacity_transition: context.manifest_capacity_transition,
            deadline: context.deadline,
            group_queue_admission: context.group_queue_admission,
            signal_profile: context.signal_profile,
            durability_policy_basis: context.durability_policy_basis,
            resources: context.resources,
            start: context.start,
            selected_segment_rewrite: false,
            source_root_generation: 0,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn mark_selected_segment_rewrite(
        mut self,
        source_root_generation: u64,
    ) -> Self {
        self.selected_segment_rewrite = true;
        self.source_root_generation = source_root_generation;
        self
    }

    pub(in crate::physical_runtime) const fn selected_segment_rewrite(&self) -> bool {
        self.selected_segment_rewrite
    }

    pub(in crate::physical_runtime) const fn source_root_generation(&self) -> u64 {
        self.source_root_generation
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.admission.mutation_identity()
    }

    pub const fn idempotency_identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.admission.idempotency_identity()
    }

    pub const fn idempotency_lease(&self) -> PhysicalMutationIdempotencyLease {
        self.admission.lease()
    }

    pub const fn request_fingerprint(&self) -> PhysicalMutationRequestFingerprint {
        self.admission.fingerprint()
    }

    pub const fn deadline(&self) -> PhysicalMutationDeadline {
        self.deadline
    }

    pub const fn group_queue_admission_tick(&self) -> PhysicalGroupQueueAdmissionTick {
        self.group_queue_admission
    }

    pub const fn signal_profile(&self) -> PhysicalSignalProfileIdentity {
        self.signal_profile
    }

    pub(in crate::physical_runtime) const fn placement(&self) -> AdmittedRecordPlacementPolicy {
        self.placement
    }

    pub(in crate::physical_runtime) const fn manifest_capacity_transition(
        &self,
    ) -> super::PhysicalManifestCapacityTransition {
        self.manifest_capacity_transition
    }

    pub fn durability_policy_basis(&self) -> PhysicalWorkSemanticBasis {
        self.durability_policy_basis.clone()
    }

    pub const fn resources(&self) -> PhysicalMutationResourceShape {
        self.resources
    }

    pub const fn disposition(&self) -> PhysicalMutationAdmissionDisposition {
        if self.admission.is_fresh() {
            PhysicalMutationAdmissionDisposition::Fresh
        } else {
            PhysicalMutationAdmissionDisposition::DuplicateUnresolved
        }
    }

    pub fn start(self) -> crate::physical_runtime::PhysicalMutationHandle {
        let start = self.start.clone();
        start.start(self)
    }

    pub fn execute(self) -> crate::physical_runtime::PhysicalMutationOutcome {
        self.start().wait()
    }

    pub(in crate::physical_runtime) const fn data_is_planned(&self) -> bool {
        matches!(self.data, PreparedPhysicalMutationData::Planned { .. })
    }

    pub(in crate::physical_runtime) fn duplicate_prepared_batch(&self) -> RecordAppendBatch {
        match &self.data {
            PreparedPhysicalMutationData::Unplanned { batch, .. }
            | PreparedPhysicalMutationData::Planned { batch, .. } => batch.duplicate_prepared(),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn materialization_observation(
        &self,
    ) -> CanonicalPayloadMaterializationObservation {
        match self.data {
            PreparedPhysicalMutationData::Unplanned {
                materialization, ..
            } => materialization,
            PreparedPhysicalMutationData::Planned { .. } => {
                unreachable!("materialization observation is consumed during data planning")
            }
        }
    }

    pub(in crate::physical_runtime::record_serving) fn attach_plans(
        mut self,
        data: PreparedPhysicalDataPlan,
        root: PreparedPhysicalRootProjection,
    ) -> Self {
        let batch = match self.data {
            PreparedPhysicalMutationData::Unplanned { batch, .. } => batch,
            PreparedPhysicalMutationData::Planned { .. } => {
                unreachable!("a prepared mutation receives one immutable data plan")
            }
        };
        self.data = PreparedPhysicalMutationData::Planned { batch, data, root };
        self
    }

    pub(in crate::physical_runtime) fn into_parts(self) -> PlannedPhysicalMutationParts {
        let PreparedPhysicalMutationData::Planned { batch, data, root } = self.data else {
            unreachable!("fresh WAL reservation requires the director-attached data plan")
        };
        PlannedPhysicalMutationParts {
            admission: self.admission,
            batch,
            data,
            root,
            context: PreparedPhysicalMutationContext {
                placement: self.placement,
                manifest_capacity_transition: self.manifest_capacity_transition,
                deadline: self.deadline,
                group_queue_admission: self.group_queue_admission,
                signal_profile: self.signal_profile,
                durability_policy_basis: self.durability_policy_basis,
                resources: self.resources,
                start: self.start,
            },
        }
    }

    pub(in crate::physical_runtime) fn from_planned_parts(
        parts: PlannedPhysicalMutationParts,
    ) -> Self {
        Self {
            admission: parts.admission,
            data: PreparedPhysicalMutationData::Planned {
                batch: parts.batch,
                data: parts.data,
                root: parts.root,
            },
            placement: parts.context.placement,
            manifest_capacity_transition: parts.context.manifest_capacity_transition,
            deadline: parts.context.deadline,
            group_queue_admission: parts.context.group_queue_admission,
            signal_profile: parts.context.signal_profile,
            durability_policy_basis: parts.context.durability_policy_basis,
            resources: parts.context.resources,
            start: parts.context.start,
            selected_segment_rewrite: false,
            source_root_generation: 0,
        }
    }
}
