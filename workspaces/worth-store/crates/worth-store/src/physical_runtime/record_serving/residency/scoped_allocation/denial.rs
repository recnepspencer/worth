use crate::physical_runtime::{
    LifecycleGeneration, PhysicalRecordPressureEvidence, PhysicalRecordResidencyFailure,
    PhysicalRecordResidencyFailureKind, PhysicalRecordResidencyFailureReason,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Store-facing denial of a successor physical-operation allocation.
///
/// The failure preserves the broad residency class, exact causal reason, and
/// pressure evidence when the lower denial represents an exhausted envelope.
/// It grants no retry or allocation authority.
pub struct PhysicalScopedAllocationFailure {
    failure: PhysicalRecordResidencyFailure,
    pressure: Option<PhysicalRecordPressureEvidence>,
}

impl PhysicalScopedAllocationFailure {
    pub(in crate::physical_runtime) fn from_denial(
        denial: worth_store_buffer_pool::PhysicalResidencyDenial,
        generation: LifecycleGeneration,
    ) -> Self {
        let failure = denial.into();
        let pressure = PhysicalRecordPressureEvidence::from_store_failure(failure, generation);
        Self { failure, pressure }
    }

    /// Returns the stable broad residency-failure class.
    pub const fn kind(self) -> PhysicalRecordResidencyFailureKind {
        self.failure.kind()
    }

    /// Returns the exact Store-owned cause of the failed admission.
    pub const fn reason(self) -> PhysicalRecordResidencyFailureReason {
        self.failure.reason()
    }

    /// Returns pressure evidence when an admitted byte envelope caused denial.
    pub const fn pressure(self) -> Option<PhysicalRecordPressureEvidence> {
        self.pressure
    }
}
