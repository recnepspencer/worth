//! Actual request and storage causes remain distinct from pure provider refusals.
use worth_execution::{ExecutionRequestArrayDenial, WorkCeilingDenial};

/// Why controlled index work stopped before producing a usable answer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexStop {
    MembershipUnavailable,
    IncompleteTraversal,
    Request(WorkCeilingDenial),
    Storage(ExecutionRequestArrayDenial),
    /// Original allocation/request refusal while retaining persistent Source material.
    MaterialStorage(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    ValueDecode(
        crate::domain_computation::primary_graph::value_decode::WorthQueryValueDecodeDenial,
    ),
    ForeignNode,
    UnwrittenNode,
    NodeCapacityExceeded,
    InvalidRoot,
    ForeignSourceSlot,
    SourceSlotUnavailable,
    CapacityOverflow,
    IncrementalUnavailable,
    SourceStructureUnavailable,
    SourceStructureIncomplete,
    DuplicateSourceSlot,
    DuplicateNodeParent,
    SourceStructureCycle,
    OrderedStorage(worth_execution::ExecutionOrderedMapDenial),
    NativeSource(worth_relational::facade::runtime::StructuralWalkDenialKind),
}

/// A provider's original typed refusal, or an original controlled-work stop.
#[derive(Debug)]
pub enum WorthQueryManagedIndexProviderStop<Domain> {
    Controlled(WorthQueryManagedIndexStop),
    Domain(Domain),
}

impl<D> From<WorthQueryManagedIndexStop> for WorthQueryManagedIndexProviderStop<D> {
    fn from(cause: WorthQueryManagedIndexStop) -> Self {
        Self::Controlled(cause)
    }
}

impl From<worth_execution::ExecutionOrderedMapDenial> for WorthQueryManagedIndexStop {
    fn from(cause: worth_execution::ExecutionOrderedMapDenial) -> Self {
        match cause {
            worth_execution::ExecutionOrderedMapDenial::Request(cause) => Self::Request(cause),
            worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => Self::CapacityOverflow,
            cause => Self::OrderedStorage(cause),
        }
    }
}
