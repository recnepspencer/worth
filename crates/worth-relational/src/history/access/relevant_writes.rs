//! Canonical authored writes, including writes whose semantic after-image is unchanged.
mod cursor;
mod maintenance;
mod managed_source;
mod page;
pub(crate) mod posting_key;
mod selection;

pub use cursor::RelationalAuthoredWriteCursor;
pub(crate) use managed_source::source_impact;
pub use managed_source::{
    RelationalAuthoredManagedSourceImpact, RelationalAuthoredManagedSourceMaterial,
    RelationalAuthoredManagedSourcePayloads,
};
pub use page::{
    RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
    RelationalAuthoredWritePageLimits, RelationalPreparedAuthoredWritePage,
};
pub use selection::{
    RelationalAuthoredEntityIndexDefinition, RelationalAuthoredManagedSourceSelector,
    RelationalAuthoredWriteScope, RelationalAuthoredWriteSelector,
};

/// A missing derived generation or canonical occurrence never proves absence.
#[derive(Debug)]
pub enum RelationalAuthoredWriteDenial {
    ForeignOwner,
    NoSelectedCommit,
    NotExactSelection,
    SourceUnavailable,
    CanonicalEvidenceUnavailable,
    GenerationUnavailable,
    UnsupportedMergeInterval,
    IncompleteIndexHistory,
    ForeignManagedSourceProgram,
    UnsupportedManagedSourceSelector,
    IncompleteManagedSourceHistory,
    IncompleteAdjacencyHistory {
        ordinal: u64,
        commit: crate::history::data::CommitId,
        relation: Option<crate::identity::data::RelationId>,
        kind: Option<crate::identity::data::KindId>,
        reason: &'static str,
    },
    UnsupportedAdjacencySelector,
    UnsupportedIndexSelector,
    NotSelectedPrefix,
    EmptySelection,
    InvalidContinuation,
    PageStorageTooSmall,
    SizeOverflow,
    BackingAllocation(std::collections::TryReserveError),
    Allocation(worth_execution::ExecutionAllocationDenial),
    OrderedStorage(worth_execution::ExecutionOrderedMapDenial),
    Request(worth_execution::WorkCeilingDenial),
    Interrupted(worth_execution::MapKernelStop),
    Retention(crate::branch::RelationalBranchBasisDenial),
    /// The original request or selected durable body could not be admitted.
    Execution(crate::execution::RelationalExecutionDenialCause),
}

impl From<worth_execution::WorkCeilingDenial> for RelationalAuthoredWriteDenial {
    fn from(cause: worth_execution::WorkCeilingDenial) -> Self {
        Self::Request(cause)
    }
}

impl From<worth_execution::ExecutionAllocationDenial> for RelationalAuthoredWriteDenial {
    fn from(cause: worth_execution::ExecutionAllocationDenial) -> Self {
        Self::Allocation(cause)
    }
}
impl From<worth_execution::ExecutionOrderedMapDenial> for RelationalAuthoredWriteDenial {
    fn from(cause: worth_execution::ExecutionOrderedMapDenial) -> Self {
        Self::OrderedStorage(cause)
    }
}
impl From<worth_execution::MapKernelStop> for RelationalAuthoredWriteDenial {
    fn from(cause: worth_execution::MapKernelStop) -> Self {
        Self::Interrupted(cause)
    }
}
impl From<worth_execution::ExecutionRequestArrayDenial> for RelationalAuthoredWriteDenial {
    fn from(cause: worth_execution::ExecutionRequestArrayDenial) -> Self {
        match cause {
            worth_execution::ExecutionRequestArrayDenial::Request(cause) => Self::Request(cause),
            worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
                Self::Allocation(cause)
            }
        }
    }
}

pub(crate) fn run<T>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    operation: impl FnOnce(
        &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<T, RelationalAuthoredWriteDenial>,
) -> Result<T, RelationalAuthoredWriteDenial> {
    let mut result = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        result = Some(operation(context));
    });
    match (outcome, result) {
        (Ok(_), Some(result)) => result,
        (
            Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )),
            Some(Err(cause)),
        ) => Err(cause),
        (Err(cause), _) => Err(RelationalAuthoredWriteDenial::Request(cause)),
        (Ok(_), None) => unreachable!("completed authored-write callback"),
    }
}

/// Admit actual traversal/copy work on the same supplied Native request.
pub(crate) fn admit_work(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    units: u64,
) -> Result<(), RelationalAuthoredWriteDenial> {
    if context.has_accounting() {
        context.checkpoint(units)?;
    } else {
        context.poll()?;
    }
    Ok(())
}
pub(crate) fn visit(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), RelationalAuthoredWriteDenial> {
    admit_work(context, 1)
}

/// AVL height is below twice its bit width: one unit per descended level.
/// Comparisons stop at the first differing byte, so key width is not priced.
pub(crate) fn ordered_visit(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    entries: usize,
) -> Result<(), RelationalAuthoredWriteDenial> {
    let work = 2 * u64::from(usize::BITS - entries.leading_zeros()) + 2;
    admit_work(context, work)
}
