//! Only freshly gathered inputs enter native retained-run advancement.
use super::*;
use crate::domain_computation::primary_graph::fact_verification_observation as observed;
use worth_execution::{ExecutionRetainedRun, RetainedRunOutcome};
struct BorrowedGather<'a, Key, Gathered>(&'a GatheredComputationPartition<Key, Gathered>);
pub(super) struct SettledPartitions<Reduced> {
    pub native_run: ExecutionRetainedRun<Reduced>,
    pub work: u64,
    pub span: u64,
}
pub(super) fn run<Key, Gathered, Reduced, Stopped>(
    partitions: &BTreeMap<PartitionIdentity, Recomputed<Key, Gathered>>,
    prior: &ExecutionRetainedRun<Reduced>,
    removed: CanonicalUniqueVec<PartitionIdentity>,
    execution: &QueryRequestExecution<'_>,
    ceiling: Option<u64>,
    kernel: &(impl Fn(
        &GatheredComputationPartition<Key, Gathered>,
        &mut MapKernelContext<'_, '_>,
    ) -> Result<Reduced, MapKernelFailure<PartitionRefusal<Stopped>>>
          + Sync),
    transport: &mut NestedFailureTransport<'_>,
) -> Result<
    SettledPartitions<Reduced>,
    MemberComputeFailure<WorthQueryPartitionedComputationDenial<Stopped>>,
>
where
    Key: Send + Sync,
    Gathered: Send + Sync,
    Reduced: Send + Sync,
    Stopped: Send,
{
    let changes = partitions
        .iter()
        .map(|(identity, partition)| {
            observed::computation_dispatch_plan();
            (*identity, BorrowedGather(&partition.value))
        })
        .collect();
    transport.call(|| {
        execution
            .dispatch()
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?
            .advance_retained(ceiling, prior, changes, removed, |input, context| {
                kernel(input.0, context)
            })
            .map_err(WorthQueryPartitionedComputationDenial::from_retained_run)
            .and_then(|outcome| {
                observed::computation_native_run(outcome.metrics());
                match outcome {
                    RetainedRunOutcome::Complete { run, report, .. } => Ok(SettledPartitions {
                        native_run: run,
                        work: report.charged_work(),
                        span: report.charged_span(),
                    }),
                    RetainedRunOutcome::Stopped { reason, .. } => Err(
                        WorthQueryPartitionedComputationDenial::from_map_stop(reason),
                    ),
                }
            })
    })
}
