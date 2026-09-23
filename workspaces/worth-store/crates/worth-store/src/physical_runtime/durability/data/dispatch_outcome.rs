use crate::physical_runtime::durability::{
    DataDispatchedPhysicalMutation, PhysicalDataEffectSettlement,
};
use crate::physical_runtime::{
    CandidateFrameContractViolation, PhysicalRecordMutationFailureEvidence,
    PhysicalRecordPressureEvidence, PhysicalRecordResidencyFailure,
    PhysicalRecordWritebackFailureEvidence, RecordAppendDenial, WalDurablePhysicalMutation,
};
use worth_store_physical_format::RecordArtifactFile;

pub enum PhysicalDataDispatchOutcome {
    Dispatched(DataDispatchedPhysicalMutation),
    RetryableAfterCleanup(CleanedPhysicalDataDispatchRetry),
    NotStarted {
        durable: WalDurablePhysicalMutation,
        cause: PhysicalDataDispatchFailureCause,
    },
    Indeterminate(IndeterminatePhysicalDataDispatch),
}

pub struct CleanedPhysicalDataDispatchRetry {
    durable: WalDurablePhysicalMutation,
    discarded_effects: Box<[PhysicalDataEffectSettlement]>,
    pressure: PhysicalRecordPressureEvidence,
    deleted_artifacts: Box<[RecordArtifactFile]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalDataDispatchFailureCause {
    PublicationAuthorityReleased,
    ForeignStore,
    StaleRuntime,
    SignalProfileMismatch,
    PhysicalPressure(PhysicalRecordPressureEvidence),
    RecordResidency(PhysicalRecordResidencyFailure),
    CandidateAdmission(RecordAppendDenial),
    CandidateFrameContract(CandidateFrameContractViolation),
    Canonical(PhysicalRecordMutationFailureEvidence),
    ExistingArtifactWriteback(PhysicalRecordWritebackFailureEvidence),
    IncompleteFrameSet,
    MissingEffectSettlement,
    /// A maintenance candidate read back from media differs from its WAL-bound
    /// bytes, so no root may name it.
    CandidateReadBackMismatch,
}

pub struct IndeterminatePhysicalDataDispatch {
    durable: WalDurablePhysicalMutation,
    effects: Vec<PhysicalDataEffectSettlement>,
    cause: PhysicalDataDispatchFailureCause,
}

impl CleanedPhysicalDataDispatchRetry {
    pub(in crate::physical_runtime) fn new(
        durable: WalDurablePhysicalMutation,
        discarded_effects: Vec<PhysicalDataEffectSettlement>,
        pressure: PhysicalRecordPressureEvidence,
        deleted_artifacts: Vec<RecordArtifactFile>,
    ) -> Self {
        Self {
            durable,
            discarded_effects: discarded_effects.into_boxed_slice(),
            pressure,
            deleted_artifacts: deleted_artifacts.into_boxed_slice(),
        }
    }

    pub const fn durable(&self) -> &WalDurablePhysicalMutation {
        &self.durable
    }

    pub fn discarded_effects(&self) -> &[PhysicalDataEffectSettlement] {
        &self.discarded_effects
    }

    pub const fn pressure(&self) -> PhysicalRecordPressureEvidence {
        self.pressure
    }

    pub fn deleted_artifacts(&self) -> &[RecordArtifactFile] {
        &self.deleted_artifacts
    }

    pub fn into_durable(self) -> WalDurablePhysicalMutation {
        self.durable
    }
}

impl IndeterminatePhysicalDataDispatch {
    pub(in crate::physical_runtime) fn new(
        durable: WalDurablePhysicalMutation,
        effects: Vec<PhysicalDataEffectSettlement>,
        cause: PhysicalDataDispatchFailureCause,
    ) -> Self {
        Self {
            durable,
            effects,
            cause,
        }
    }

    pub const fn mutation_identity(&self) -> crate::physical_runtime::PhysicalMutationIdentity {
        self.durable.mutation_identity()
    }

    pub fn completed_frames(&self) -> usize {
        self.effects.len()
    }

    pub const fn durable(&self) -> &WalDurablePhysicalMutation {
        &self.durable
    }

    pub fn effects(&self) -> &[PhysicalDataEffectSettlement] {
        &self.effects
    }

    pub const fn cause(&self) -> &PhysicalDataDispatchFailureCause {
        &self.cause
    }
}
