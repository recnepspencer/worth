use super::super::{construction::SubtreeResult, value_storage::ValueStorage};
use super::*;
use crate::backend::{run_native_batch_with_charge, BatchOutcome};

pub(super) trait ScheduledValues<T>: ValueStorage<T> + Sync {
    fn batch<F: Fn(&T, &T) -> T>(
        &self,
        tasks: &[SubtreeSpec],
    ) -> Result<AdmittedBatch<ScheduledTask>, ScheduledReductionError>;
    fn run<Kernel>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        batch: &AdmittedBatch<ScheduledTask>,
        backend: BackendKind,
        kernel: &Kernel,
    ) -> BatchOutcome<SubtreeResult<T>, ReductionRunFailure<KernelStop>>
    where
        Kernel: Fn(
                &ScheduledTask,
                &mut KernelContext<'_, '_>,
            )
                -> Result<SubtreeResult<T>, KernelFailure<ReductionRunFailure<KernelStop>>>
            + Sync;
}

impl<T: Clone + Send + Sync + ChargedBytes + CanonicalBits> ScheduledValues<T> for BoundedValues {
    fn batch<F: Fn(&T, &T) -> T>(
        &self,
        tasks: &[SubtreeSpec],
    ) -> Result<AdmittedBatch<ScheduledTask>, ScheduledReductionError> {
        let scratch = self.0.checked_mul(6).ok_or_else(|| {
            ScheduledReductionError::Admission(LeaseDenial::charged_bytes_overflow())
        })?;
        let entries = tasks
            .iter()
            .copied()
            .map(|spec| {
                let capacity = ReductionTree::<T, F>::checked_subtree_result_bound(
                    spec.len(),
                    self.0,
                )
                .ok_or_else(|| {
                    ScheduledReductionError::Admission(LeaseDenial::charged_bytes_overflow())
                })?;
                Ok((
                    spec.identity,
                    ScheduledTask {
                        spec,
                        completed: Some(OnceLock::new()),
                    },
                    scratch,
                    capacity,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        AdmittedBatch::try_admit(entries, 0).map_err(|denial| match denial {
            BatchDenial::Identities => identities(ReductionMetrics::default()),
            BatchDenial::MemoryOverflow => {
                ScheduledReductionError::Admission(LeaseDenial::charged_bytes_overflow())
            }
        })
    }
    fn run<Kernel>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        batch: &AdmittedBatch<ScheduledTask>,
        backend: BackendKind,
        kernel: &Kernel,
    ) -> BatchOutcome<SubtreeResult<T>, ReductionRunFailure<KernelStop>>
    where
        Kernel: Fn(
                &ScheduledTask,
                &mut KernelContext<'_, '_>,
            )
                -> Result<SubtreeResult<T>, KernelFailure<ReductionRunFailure<KernelStop>>>
            + Sync,
    {
        run_checked_batch_with_charge(Some(lease), batch, backend, kernel, false)
    }
}

impl<T: Send + Sync + CanonicalBits> ScheduledValues<T> for NativeValues<'_, '_> {
    fn batch<F: Fn(&T, &T) -> T>(
        &self,
        tasks: &[SubtreeSpec],
    ) -> Result<AdmittedBatch<ScheduledTask>, ScheduledReductionError> {
        let metrics = ReductionMetrics::for_mode(<Self as ValueStorage<T>>::has_accounting(self));
        let mut ids = Vec::new();
        let mut values = Vec::new();
        ids.try_reserve_exact(tasks.len())
            .map_err(|_| allocation(metrics))?;
        values
            .try_reserve_exact(tasks.len())
            .map_err(|_| allocation(metrics))?;
        for &spec in tasks {
            ids.push(spec.identity);
            values.push(ScheduledTask {
                spec,
                completed: <Self as ValueStorage<T>>::has_accounting(self).then(OnceLock::new),
            });
        }
        let ids = worth_proof::CanonicalUniqueVec::try_from_sorted_unique(ids)
            .map_err(|_| identities(metrics))?;
        Ok(AdmittedBatch::admit_native(ids, values))
    }
    fn run<Kernel>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        batch: &AdmittedBatch<ScheduledTask>,
        backend: BackendKind,
        kernel: &Kernel,
    ) -> BatchOutcome<SubtreeResult<T>, ReductionRunFailure<KernelStop>>
    where
        Kernel: Fn(
                &ScheduledTask,
                &mut KernelContext<'_, '_>,
            )
                -> Result<SubtreeResult<T>, KernelFailure<ReductionRunFailure<KernelStop>>>
            + Sync,
    {
        run_native_batch_with_charge(Some(lease), batch, backend, kernel, false)
    }
}

fn identities(metrics: ReductionMetrics) -> ScheduledReductionError {
    ScheduledReductionError::Reduction(ReductionRunFailure {
        reason: ReductionRunStop::Denial(ReductionDenial::IdentitiesNotCanonical),
        metrics,
    })
}
fn allocation(metrics: ReductionMetrics) -> ScheduledReductionError {
    ScheduledReductionError::Reduction(ReductionRunFailure {
        reason: ReductionRunStop::Denial(ReductionDenial::Allocation(
            crate::ExecutionPhysicalAllocationDenialKind::Allocator,
        )),
        metrics,
    })
}
