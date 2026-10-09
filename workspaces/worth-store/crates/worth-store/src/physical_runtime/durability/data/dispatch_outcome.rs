use crate::physical_runtime::durability::{
    DataDispatchedPhysicalMutation, PhysicalDataEffectSettlement,
};
use crate::physical_runtime::{
    CandidateFrameContractViolation, PhysicalRecordMutationFailureEvidence,
    PhysicalRecordPressureEvidence, PhysicalRecordResidencyFailure,
    PhysicalRecordWritebackFailureEvidence, RecordAppendDenial, WalDurablePhysicalMutation,
};

pub enum PhysicalDataDispatchOutcome {
    Dispatched(DataDispatchedPhysicalMutation),
    Suspended(SuspendedPhysicalDataDispatch),
    NotStarted {
        durable: WalDurablePhysicalMutation,
        cause: PhysicalDataDispatchFailureCause,
    },
    Indeterminate(IndeterminatePhysicalDataDispatch),
}

/// Settled prefix retained under the original WAL-bound mutation and range
/// claims. Resuming never deletes shared artifacts or reissues completed work.
pub struct SuspendedPhysicalDataDispatch {
    durable: WalDurablePhysicalMutation,
    completed_effects: Vec<PhysicalDataEffectSettlement>,
    cause: PhysicalDataDispatchFailureCause,
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

impl SuspendedPhysicalDataDispatch {
    pub(in crate::physical_runtime) fn new(
        durable: WalDurablePhysicalMutation,
        completed_effects: Vec<PhysicalDataEffectSettlement>,
        cause: PhysicalDataDispatchFailureCause,
    ) -> Self {
        Self {
            durable,
            completed_effects,
            cause,
        }
    }

    pub const fn durable(&self) -> &WalDurablePhysicalMutation {
        &self.durable
    }

    pub fn completed_effects(&self) -> &[PhysicalDataEffectSettlement] {
        &self.completed_effects
    }

    pub const fn cause(&self) -> &PhysicalDataDispatchFailureCause {
        &self.cause
    }

    pub fn into_durable(mut self) -> WalDurablePhysicalMutation {
        self.durable
            .retain_completed_data_prefix(self.completed_effects);
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
