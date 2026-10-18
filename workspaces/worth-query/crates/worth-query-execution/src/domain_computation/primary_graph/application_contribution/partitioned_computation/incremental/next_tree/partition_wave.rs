//! Carried and recomputed kernels settle through one independent map.
use super::*;
use worth_execution::ChargedBytes;

struct BorrowedPartition<'a, Key, Gathered> {
    disposition: &'a Disposition<Key, Gathered>,
}
impl<Key, Gathered> ChargedBytes for BorrowedPartition<'_, Key, Gathered> {
    fn additional_charged_bytes(&self) -> u64 {
        // GatheredMemory and the prior already hold the borrowed data.
        0
    }
}

enum SettledPartition<Reduced> {
    Carried,
    Computed(Reduced),
}
impl<Reduced: ChargedBytes> ChargedBytes for SettledPartition<Reduced> {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Carried => 0,
            Self::Computed(value) => value.additional_charged_bytes(),
        }
    }
}

pub(super) struct SettledPartitions<Reduced> {
    pub values: BTreeMap<PartitionIdentity, Reduced>,
    pub work: u64,
    pub span: u64,
}

pub(super) fn run<Key, Item, Gathered, Reduced, Stopped>(
    dispositions: &BTreeMap<PartitionIdentity, Disposition<Key, Gathered>>,
    _retained: &RetainedPartitions<Key, Item, Reduced>,
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
    let map = ExecutionMap::<_, u64>::from_keyless_partitions(
        dispositions
            .iter()
            .map(|(identity, disposition)| {
                (
                    *identity,
                    KeylessPartition {
                        value: BorrowedPartition { disposition },
                        kernel_scratch_bytes: 0,
                        max_result_bytes: match disposition {
                            Disposition::Carried(_) => 0,
                            Disposition::Recomputed(_) => declared_bytes,
                        },
                    },
                )
            })
            .collect(),
    )
    .map_err(WorthQueryPartitionedComputationDenial::from_map_overflow)?;
    let outcome = transport.call(|| {
        execution
            .dispatch()
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?
            .map(
                ceiling,
                &map,
                execution
                    .reserve(0)
                    .map_err(WorthQueryPartitionedComputationDenial::Resource)?,
                |input, context| match input.disposition {
                    Disposition::Carried(partition) => {
                        context.carry_kernel(&partition.kernel_receipt)?;
                        Ok(SettledPartition::Carried)
                    }
                    Disposition::Recomputed(marked) => {
                        kernel(&marked.value, context).map(SettledPartition::Computed)
                    }
                },
            )
            .map_err(|denial| {
                WorthQueryPartitionedComputationDenial::from_work_ceiling(
                    denial,
                    WorthQueryPartitionedComputationDenial::ReducerPanicked,
                )
            })
            .and_then(|outcome| match outcome {
                MapOutcome::Complete { values, report } => Ok((values, report)),
                MapOutcome::Stopped { reason, .. } => Err(
                    WorthQueryPartitionedComputationDenial::from_map_stop(reason),
                ),
            })
    })?;
    Ok(SettledPartitions {
        values: map
            .identities()
            .iter()
            .copied()
            .zip(outcome.0)
            .filter_map(|(identity, value)| match value {
                SettledPartition::Carried => None,
                SettledPartition::Computed(value) => Some((identity, value)),
            })
            .collect(),
        work: outcome.1.charged_work(),
        span: outcome.1.charged_span(),
    })
}
