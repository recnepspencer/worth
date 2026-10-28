//! Canonical authored writes, including writes whose semantic after-image is unchanged.
mod cursor;
mod maintenance;
mod page;
mod selection;

pub use cursor::RelationalAuthoredWriteCursor;
pub use page::{
    RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
    RelationalAuthoredWritePageLimits, RelationalPreparedAuthoredWritePage,
};
pub use selection::{
    RelationalAuthoredEntityIndexDefinition, RelationalAuthoredWriteScope,
    RelationalAuthoredWriteSelector,
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

pub(crate) fn visit(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), RelationalAuthoredWriteDenial> {
    if context.has_accounting() {
        context.checkpoint(1)?;
    } else {
        context.poll()?;
    }
    Ok(())
}

/// AVL height is below twice its bit width. Pay key comparisons before the
/// persistent backing walks; memory admission remains the backing owner's job.
pub(crate) fn ordered_visit(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    entries: usize,
    key_bytes: u64,
) -> Result<(), RelationalAuthoredWriteDenial> {
    let height = 2 * u64::from(usize::BITS - entries.leading_zeros()) + 2;
    let work =
        height
            .checked_mul(key_bytes.max(1))
            .ok_or(RelationalAuthoredWriteDenial::Interrupted(
                worth_execution::MapKernelStop::WorkCounterOverflow,
            ))?;
    if context.has_accounting() {
        context.checkpoint(work)?;
    } else {
        context.poll()?;
    }
    Ok(())
}
