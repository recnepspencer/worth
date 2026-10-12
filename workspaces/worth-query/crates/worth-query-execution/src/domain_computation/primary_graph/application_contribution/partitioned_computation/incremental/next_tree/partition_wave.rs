//! Carried and recomputed kernels settle through one independent map.
use super::*;
use std::sync::Mutex;
use worth_execution::ChargedBytes;

struct BorrowedPartition<'a, Key, Gathered> {
    disposition: &'a Disposition<Key, Gathered>,
    identity: PartitionIdentity,
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
    pub costs: BTreeMap<PartitionIdentity, (u64, u64)>,
    pub work: u64,
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
                        value: BorrowedPartition {
                            disposition,
                            identity: *identity,
                        },
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
    let costs = Mutex::new(BTreeMap::new());
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
                |input, context| {
                    let result = match input.disposition {
                        Disposition::Carried(partition) => {
                            if partition.kernel_units <= context.remaining_work() {
                                context.checkpoint_cost(
                                    partition.kernel_units,
                                    partition.kernel_span,
                                )?;
                            } else {
                                // The full kernel's accepted steps run in its original
                                // canonical slot. Settlement owns the global prefix;
                                // no width-dependent second ledger selects this slot.
                                partition.kernel_steps.replay(context)?;
                            }
                            Ok(SettledPartition::Carried)
                        }
                        Disposition::Recomputed(marked) => {
                            kernel(&marked.value, context).map(SettledPartition::Computed)
                        }
                    };
                    costs
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .insert(input.identity, context.charged_cost());
                    result
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
        costs: costs
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        work: outcome.1.charged_work(),
    })
}
