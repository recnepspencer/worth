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
pub use lease::{
    ConstructionDenial, ExecutionAllocationDenial, ExecutionAllocationDenialClass,
    ExecutionAllocationDenialKind, ExecutionAllocationPolicy, ExecutionArray,
    ExecutionArrayBuilder, ExecutionArrayIntoIter, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionByteBuffer, ExecutionImmutableBytes, ExecutionLeaseStatus,
    ExecutionPhysicalAllocationDenialKind, ExecutionPolicyDenial, ExecutionResourceLease,
    LeaseDenial, LeaseRequest,
};
pub use memory::{
    ExecutionMemoryReservation, ExecutionResultMemory, MemoryLimitDenial, MemoryLimitLevel,
    SerialMemoryBudget,
};
pub use request::ExecutionRequest;
pub use serial::SerialRequest;

pub(crate) use request::consult_observation;
