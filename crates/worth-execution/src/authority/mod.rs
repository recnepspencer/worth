mod cancellation;
mod equivalence;
mod lease;
mod memory;
mod request;
mod serial;

pub use cancellation::{CancellationSource, CancellationToken};
pub use equivalence::EquivalencePredicate;
pub(crate) use lease::ResourceReservation;
#[cfg(test)]
pub(crate) use lease::SlotRefusal;
pub(crate) use lease::{
    interruption_backing_lease, unrelated_interruption_context, InterruptionViewBuilder,
};
pub use lease::{
    ConstructionDenial, ExecutionAllocationDenial, ExecutionAllocationDenialClass,
    ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder, ExecutionArrayIntoIter,
    ExecutionAuthority, ExecutionAuthorityConfig, ExecutionByteBuffer, ExecutionImmutableBytes,
    ExecutionInterruptionView, ExecutionInterruptionViewDenial, ExecutionLeaseStatus,
    ExecutionPhysicalAllocationDenialKind, ExecutionPolicyDenial, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial, ExecutionResourceLease, LeaseDenial, LeaseRequest,
};
pub use memory::{
    ExecutionMemoryReservation, ExecutionResultMemory, MemoryLimitDenial, MemoryLimitLevel,
    SerialMemoryBudget,
};
pub use request::ExecutionRequest;
pub use serial::SerialRequest;

pub(crate) use request::consult_observation;

#[cfg(test)]
pub(crate) use lease::ExecutionAllocationDenialKind;
