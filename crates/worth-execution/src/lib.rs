#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Bounded computation authority shared by WORTH runtimes.
//!
//! Only the composition root constructs an authority. Domain runtimes receive
//! leases and use checked patterns; they do not control the physical backend.

mod execution_stack;
pub use execution_stack::run_with_execution_stack;

mod monotonic_clock;
mod monotonic_deadline;
pub use monotonic_clock::{
    MonotonicClock, MonotonicTimeAdvanceDenial, MonotonicTimeConstructionDenial,
    MonotonicTimeController,
};
pub use monotonic_deadline::MonotonicDeadline;

mod authority;
mod backend;
mod oracle;
mod partition;
mod pattern;
mod reduction;
pub mod report;
mod storage;
pub mod time;
pub mod workers;
pub use storage::{
    build_ordered_tree, build_sorted_ordered_tree, insert_ordered_tree, remove_ordered_tree,
    ExecutionOrderedMap, ExecutionOrderedMapCell, ExecutionOrderedMapDenial,
    ExecutionOrderedMapUniqueDenial, ExecutionOrderedTreeBounds, ExecutionOrderedTreeNode,
    ExecutionOrderedTreeStorage,
};

pub use authority::{
    CancellationSource, CancellationToken, ConstructionDenial, EquivalencePredicate,
    ExecutionAllocationDenial, ExecutionAllocationDenialClass, ExecutionAllocationPolicy,
    ExecutionArray, ExecutionArrayBuilder, ExecutionArrayIntoIter, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionByteBuffer, ExecutionImmutableBytes,
    ExecutionInterruptionView, ExecutionInterruptionViewDenial, ExecutionLeaseStatus,
    ExecutionMemoryReservation, ExecutionPhysicalAllocationDenialKind, ExecutionPolicyDenial,
    ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial,
    ExecutionResourceLease, ExecutionResultMemory, LeaseDenial, LeaseRequest, MemoryLimitDenial,
    MemoryLimitLevel, SerialMemoryBudget, SerialRequest,
};
pub use oracle::{compare_canonical_values, CanonicalBits};

#[cfg(test)]
mod tests;
pub use backend::{
    ExecutionKernelReceipt, ExecutionKernelReceiptStorage, ExecutionOwnerContext,
    ExecutionRetainedResult, ExecutionWorkLoan, ExecutionWorkLoanDenial, NestedComputationStop,
    ScopeOutcome, ScopeStop,
};
pub use partition::{
    Bisection, BisectionDenial, BisectionQuality, ComponentDenial, ComponentEdit,
    ComponentPartitioner, KeyedDenial, KeyedEditDenial, KeyedItem, KeyedPartitioner,
    PartitionItemId, PartitionMembers, PartitionUpdateDenial, PartitionWork, WeightedEdge,
    WeightedItem,
};
pub use pattern::{
    BackInput, DecomposeCertificationFailure, DecomposeComplete, DecomposeFailure,
    DecomposeInputDenial, DecomposeKernelEditions, DecomposeReuse, DecomposeRunFailure,
    DecomposeStage, ExecutionDecompose, ExecutionForkJoin, ExecutionMap, ExecutionReduce,
    ExecutionRetainedPrefix, ExecutionRetainedRun, ExecutionRounds, ExecutionScan,
    ExecutionWorkCeiling, ForkChild, ForkJoinDenial, ForkJoinOutcome, InterfaceSolution,
    InteriorResult, KeylessPartition, MapDenial, MapKernelContext, MapKernelFailure, MapKernelStop,
    MapMemoryOverflow, MapOutcome, MapPartition, MapStop, MeteredPayload, NativePayload,
    NativeRetainedReduceOutcome, NativeRetainedReduction, OracleMismatch, PreparedExecutionMap,
    ReduceCertificationFailure, ReduceInputDenial, RetainedMapDenial, RetainedMapInput,
    RetainedReduceDenial, RetainedReduceOutcome, RetainedReduction, RetainedRunDenial,
    RetainedRunMetrics, RetainedRunOutcome, RoundsDenial, RoundsOutcome, ScanDenial, ScanOutcome,
    WorkCeilingDenial,
};
pub use reduction::{
    NativeReductionTree, ReductionAccounting, ReductionDenial, ReductionEditMemoryBoundError,
    ReductionInsertMemoryBoundError, ReductionMetrics, ReductionPlan, ReductionRunFailure,
    ReductionRunStop, ReductionShapeCostDenial, ReductionTree,
};
pub use report::{
    DenialClass, DiagnosticDetail, DiagnosticSink, Disposition, OperatorReport, ReportCode,
    ReportProjection, Reportable, RetryHint,
};
pub use worth_foundational::ChargedBytes;

pub use authority::consult_observation::{
    with_request_consult_observer_for_test, RequestConsultObservationForTest,
};
#[doc(hidden)]
pub use backend::ExecutionCheckpointObservationForTest;
#[doc(hidden)]
pub use backend::{ExecutionScopeSetupCountsForTest, ExecutionScopeSetupObservationForTest};

#[cfg(test)]
pub(crate) use authority::ExecutionAllocationDenialKind;

mod framing;
pub use framing::{
    charge_framed_bytes, decode_framed_envelope, encode_framed_array, encode_framed_bytes,
    encode_framed_envelope, fill_admitted_framed_buffer, inspect_framed_envelope,
    write_framed_envelope, FramedEnvelopeError, FramedEnvelopeSpec, FRAMED_ENVELOPE_HEADER_BYTES,
};

/// The original execution request's admission or in-flight stop.
pub type ExecutionStop = WorkCeilingDenial;

mod cooperative_retirement;
pub use cooperative_retirement::{
    drain_cooperative_retirement, try_lock_retirement, try_retirement_guard, CooperativeRetireTask,
    CooperativeRetirementLockDenial, CooperativeRetirementState, CooperativeStep,
};
