use crate::physical_runtime::{
    IndeterminatePhysicalMutation, PhysicalMutationIdempotencyKeyIdentity,
    PhysicalMutationIdentity, PhysicalMutationIndeterminateStage,
    PhysicalMutationPreSealAdmissionDetail, PhysicalMutationProvenNoEffectCause,
    PhysicalMutationRequestFingerprint, ProvenNoEffectPhysicalMutation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenNoEffectPhysicalMutationEvidence {
    mutation: PhysicalMutationIdentity,
    idempotency: PhysicalMutationIdempotencyKeyIdentity,
    fingerprint: PhysicalMutationRequestFingerprint,
    cause: PhysicalMutationProvenNoEffectCause,
    admission_detail: Option<PhysicalMutationPreSealAdmissionDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminatePhysicalMutationEvidence {
    mutation: PhysicalMutationIdentity,
    idempotency: PhysicalMutationIdempotencyKeyIdentity,
    fingerprint: PhysicalMutationRequestFingerprint,
    stage: PhysicalMutationIndeterminateStage,
    completed_effects: u32,
    root_preparation_failure:
        Option<crate::physical_runtime::PhysicalMutationRootPreparationFailure>,
}

impl ProvenNoEffectPhysicalMutationEvidence {
    pub(in crate::physical_runtime) fn from_fate(fate: &ProvenNoEffectPhysicalMutation) -> Self {
        Self {
            mutation: fate.mutation_identity(),
            idempotency: fate.idempotency_identity(),
            fingerprint: fate.request_fingerprint(),
            cause: fate.cause(),
            admission_detail: fate.admission_detail().cloned(),
        }
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.mutation
    }
    pub const fn idempotency_identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.idempotency
    }
    pub const fn request_fingerprint(&self) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }
    pub const fn cause(&self) -> PhysicalMutationProvenNoEffectCause {
        self.cause
    }

    pub const fn admission_detail(&self) -> Option<&PhysicalMutationPreSealAdmissionDetail> {
        self.admission_detail.as_ref()
    }
}

impl IndeterminatePhysicalMutationEvidence {
    pub(in crate::physical_runtime) fn from_fate(fate: &IndeterminatePhysicalMutation) -> Self {
        Self {
            mutation: fate.mutation_identity(),
            idempotency: fate.idempotency_identity(),
            fingerprint: fate.request_fingerprint(),
            stage: fate.stage(),
            completed_effects: fate.completed_effect_count(),
            root_preparation_failure: fate.root_preparation_failure().cloned(),
        }
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.mutation
    }
    pub const fn idempotency_identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.idempotency
    }
    pub const fn request_fingerprint(&self) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }
    pub const fn stage(&self) -> PhysicalMutationIndeterminateStage {
        self.stage
    }
    pub const fn completed_effect_count(&self) -> u32 {
        self.completed_effects
    }

    pub fn root_preparation_failure(
        &self,
    ) -> Option<&crate::physical_runtime::PhysicalMutationRootPreparationFailure> {
        self.root_preparation_failure.as_ref()
    }
}
