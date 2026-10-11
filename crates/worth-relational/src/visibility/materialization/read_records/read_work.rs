use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};
use worth_execution::{
    ChargedBytes, ExecutionScan, MapKernelContext, MapKernelFailure, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

struct MaterializedRead<T> {
    value: T,
    owned_bytes: u64,
}

impl<T> ChargedBytes for MaterializedRead<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Run the native scan's checkpoints in the caller's active request. Native
/// limits stay in the value; request stops return before any value can escape.
pub(super) fn metered<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(&mut MapKernelContext<'_, '_>) -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    super::read_request::consult(execution)?;
    execution
        .in_scope(|lease| {
            let identity = PartitionIdentity::new(1);
            let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())])
                .expect("one canonical read partition");
            let outcome = scan.run::<(), _, (), _>(
                lease,
                (),
                0,
                execution.memory_limit() / 4,
                0,
                0,
                |_, _, context| {
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    let value = read(context);
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    let owned_bytes = owned_bytes(&value);
                    Ok(((), MaterializedRead { value, owned_bytes }))
                },
            );
            match outcome {
                ScanOutcome::Complete { mut prefixes, .. } => {
                    Ok(prefixes.pop().expect("one materialized read").value)
                }
                ScanOutcome::Stopped { reason, report, .. } => {
                    match (crate::runtime::QueryReadExecutionStop::PreparationStopped {
                        reason,
                        report,
                    })
                    .execution_denial()
                    {
                        Some(CommitExecutionDenial {
                            kind: CommitExecutionDenialKind::Cause(cause),
                            ..
                        }) => Err(cause),
                        None => unreachable!("a read meter emits only request stops"),
                    }
                }
            }
        })
        .map_err(super::read_request::cause)?
}

/// A keyed record read uses the same one-unit checkpoint as its streamed
/// neighbor, before touching or cloning the record.
pub(super) fn point<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut() -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    metered(
        execution,
        |context| {
            context.checkpoint(1).ok()?;
            Some(read())
        },
        |value| value.as_ref().map_or(0, &owned_bytes),
    )?
    .ok_or(Cause::WorkExhausted)
}
