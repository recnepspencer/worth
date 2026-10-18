//! Actual retained values and recomputed kernels settle in the native owner.
use super::*;
use crate::domain_computation::primary_graph::fact_verification_observation as observed;
use worth_execution::{ChargedBytes, ExecutionRetainedResult, RetainedMapInput};

struct BorrowedGather<'a, Key, Gathered>(&'a GatheredComputationPartition<Key, Gathered>);
impl<Key, Gathered> ChargedBytes for BorrowedGather<'_, Key, Gathered> {
    fn additional_charged_bytes(&self) -> u64 {
        0
    } // Existing gather custody holds the referent.
}

pub(super) struct SettledPartitions<Reduced> {
    pub values: BTreeMap<PartitionIdentity, ExecutionRetainedResult<Reduced>>,
    pub work: u64,
    pub span: u64,
}

pub(super) fn run<Key, Gathered, Reduced, Stopped>(
    dispositions: &BTreeMap<PartitionIdentity, Disposition<Key, Gathered, Reduced>>,
    execution: &QueryRequestExecution<'_>,
    ceiling: u64,
    declared_bytes: u64,
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
    Gathered: Send + Sync + ChargedBytes,
    Reduced: Clone + Send + Sync + ChargedBytes + worth_execution::CanonicalBits,
    Stopped: Send + ChargedBytes,
{
    // This supplied roster is deliberately still full. The native owner, not
    // Query cost replay, executes every carried slot and canonical prefix.
    let map = ExecutionMap::<_, u64>::from_keyless_partitions(
        dispositions
            .iter()
            .map(|(identity, disposition)| {
                observed::computation_dispatch_plan();
                let (value, max_result_bytes) = match disposition {
                    Disposition::Carried(partition) => (
                        RetainedMapInput::Carry(partition.kernel_result.clone()),
                        partition.kernel_result.additional_charged_bytes(),
                    ),
                    Disposition::Recomputed(marked) => (
                        RetainedMapInput::Evaluate(BorrowedGather(&marked.value)),
                        declared_bytes,
                    ),
                };
                (
                    *identity,
                    KeylessPartition {
                        value,
                        kernel_scratch_bytes: 0,
                        max_result_bytes,
                    },
                )
            })
            .collect(),
    )
    .map_err(WorthQueryPartitionedComputationDenial::from_map_overflow)?;
    let (values, report) = transport.call(|| {
        execution
            .dispatch()
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?
            .map_retained(
                ceiling,
                &map,
                execution
                    .reserve(0)
                    .map_err(WorthQueryPartitionedComputationDenial::Resource)?,
                |input, context| kernel(input.0, context),
            )
            .map_err(WorthQueryPartitionedComputationDenial::from_retained_map)
            .and_then(|outcome| match outcome {
                MapOutcome::Complete { values, report } => Ok((values, report)),
                MapOutcome::Stopped { reason, .. } => Err(
                    WorthQueryPartitionedComputationDenial::from_map_stop(reason),
                ),
            })
    })?;
    Ok(SettledPartitions {
        values: dispositions
            .iter()
            .zip(values)
            .filter_map(|((identity, disposition), value)| {
                observed::computation_result_receiver();
                matches!(disposition, Disposition::Recomputed(_)).then_some((*identity, value))
            })
            .collect(),
        work: report.charged_work(),
        span: report.charged_span(),
    })
}
