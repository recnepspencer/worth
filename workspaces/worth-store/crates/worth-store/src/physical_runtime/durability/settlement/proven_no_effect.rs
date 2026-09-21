use crate::physical_runtime::{
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdentity,
    PhysicalMutationRequestFingerprint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationProvenNoEffectCause {
    CancelledBeforeGroupSeal,
    DeadlineElapsedBeforeGroupSeal,
    WorkerUnavailableBeforeGroupSeal,
    AdmissionDeniedBeforeGroupSeal,
    /// Another root-changing publication is already pending, so this attempt
    /// takes no reservation and performs no WAL or data effect.
    ScopeConflict,
    /// A published predecessor advanced the root after this rewrite was planned.
    SourceChanged,
}

impl PhysicalMutationProvenNoEffectCause {
    pub(in crate::physical_runtime) const fn encoding_code(self) -> u8 {
        match self {
            Self::CancelledBeforeGroupSeal => 1,
            Self::DeadlineElapsedBeforeGroupSeal => 2,
            Self::WorkerUnavailableBeforeGroupSeal => 3,
            Self::AdmissionDeniedBeforeGroupSeal => 4,
            Self::ScopeConflict => 5,
            Self::SourceChanged => 6,
        }
    }

    pub(in crate::physical_runtime) const fn decode(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::CancelledBeforeGroupSeal),
            2 => Some(Self::DeadlineElapsedBeforeGroupSeal),
            3 => Some(Self::WorkerUnavailableBeforeGroupSeal),
            4 => Some(Self::AdmissionDeniedBeforeGroupSeal),
            5 => Some(Self::ScopeConflict),
            6 => Some(Self::SourceChanged),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvenNoEffectPhysicalMutation {
    idempotency: PhysicalMutationIdempotencyKeyIdentity,
    fingerprint: PhysicalMutationRequestFingerprint,
    mutation: PhysicalMutationIdentity,
    cause: PhysicalMutationProvenNoEffectCause,
}

impl ProvenNoEffectPhysicalMutation {
    pub(in crate::physical_runtime) const fn before_group_seal(
        idempotency: PhysicalMutationIdempotencyKeyIdentity,
        fingerprint: PhysicalMutationRequestFingerprint,
        mutation: PhysicalMutationIdentity,
        cause: PhysicalMutationProvenNoEffectCause,
    ) -> Self {
        Self {
            idempotency,
            fingerprint,
            mutation,
            cause,
        }
    }

    pub const fn idempotency_identity(self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.idempotency
    }

    pub const fn request_fingerprint(self) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }

    pub const fn mutation_identity(self) -> PhysicalMutationIdentity {
        self.mutation
    }

    pub const fn cause(self) -> PhysicalMutationProvenNoEffectCause {
        self.cause
    }

    pub const fn diagnostic_evidence(
        self,
    ) -> crate::physical_runtime::ProvenNoEffectPhysicalMutationEvidence {
        crate::physical_runtime::ProvenNoEffectPhysicalMutationEvidence::from_fate(self)
    }
}
