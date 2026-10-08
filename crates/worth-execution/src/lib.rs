//! Bounded computation authority shared by WORTH runtimes.
//!
//! Only the composition root constructs an authority. Domain runtimes receive
//! leases and use checked patterns; they do not control the physical backend.

mod authority;
mod backend;
mod oracle;
mod partition;
mod pattern;
mod reduction;
mod report;

pub use authority::{
    CancellationToken, ConstructionDenial, EquivalencePredicate, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionByteAllocationDenial, ExecutionByteAllocationDenialKind,
    ExecutionByteAllocationPolicy, ExecutionByteBuffer, ExecutionImmutableBytes,
    ExecutionLeaseStatus, ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial,
    LeaseRequest,
};
pub use oracle::{compare_canonical_values, CanonicalBits};

#[cfg(test)]
mod tests;
pub use partition::{
    Bisection, BisectionDenial, BisectionQuality, ComponentDenial, ComponentPartitioner,
    KeyedDenial, KeyedItem, KeyedPartitioner, PartitionItemId, PartitionRoute,
    PartitionUpdateDenial, PartitionWork, SourceFactId, WeightedEdge, WeightedItem,
};
pub use pattern::{
    BackInput, DecomposeCertificationFailure, DecomposeComplete, DecomposeFailure,
    DecomposeInputDenial, DecomposeKernelEditions, DecomposeReuse, DecomposeRunFailure,
    DecomposeStage, ExecutionDecompose, ExecutionForkJoin, ExecutionMap, ExecutionReduce,
    ExecutionRounds, ExecutionScan, ForkChild, ForkJoinDenial, ForkJoinOutcome, InterfaceSolution,
    InteriorResult, MapDenial, MapKernelContext, MapKernelFailure, MapKernelStop, MapOutcome,
    MapPartition, MapStop, OracleMismatch, PreparedExecutionMap, ReduceCertificationFailure,
    ReduceInputDenial, RoundsDenial, RoundsOutcome, ScanDenial, ScanOutcome,
};
pub use reduction::{
    ReductionDenial, ReductionMetrics, ReductionPlan, ReductionRunFailure, ReductionRunStop,
    ReductionTree,
};
pub use report::ChargedBytes;
