//! Moved whole from `domain_computation::primary_graph::vocabulary::application_query::stop`: the items below and every impl they own.
use worth_execution::ExecutionRequestArrayDenial;
use worth_execution::WorkCeilingDenial;

/// Why controlled index work stopped before producing a usable answer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexStop {
    Projection(crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenial),
    Content(worth_relational::facade::content::RelationalContentDenial),
    Coverage(worth_proof::ManagedIndexCoverageDenial),
    ForeignCoverage,
    IncompleteCoverage,
    MembershipUnavailable,
    RefinementUnavailable,
    IncompleteRefinement,
    IncompleteTraversal,
    SummaryUnavailable,
    SummaryMismatch,
    ForeignSummarySource,
    Request(WorkCeilingDenial),
    Storage(ExecutionRequestArrayDenial),
    /// Original allocation/request refusal while retaining persistent Source material.
    MaterialStorage(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    ValueDecode(
        crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial,
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

impl From<worth_execution::ExecutionOrderedMapDenial> for WorthQueryManagedIndexStop {
    fn from(cause: worth_execution::ExecutionOrderedMapDenial) -> Self {
        match cause {
            worth_execution::ExecutionOrderedMapDenial::Request(cause) => Self::Request(cause),
            worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => Self::CapacityOverflow,
            cause => Self::OrderedStorage(cause),
        }
    }
}
