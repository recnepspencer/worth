use std::marker::PhantomData;

use worth_foundational::ExecutionPosture;

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial},
    backend::{
        prepare_batch_resources, run_checked_batch_prepared, BackendKind, PreparedBatchResources,
    },
    report::ChargedBytes,
};

use super::{ExecutionMap, MapKernelContext, MapKernelFailure, MapOutcome};

/// A checked map with its exact memory admission held across evaluator work.
/// It is single use and bound to the active request context at preparation.
pub struct PreparedExecutionMap<'authority, T, K, R, E> {
    map: ExecutionMap<T, K>,
    lease: ExecutionResourceLease<'authority>,
    resources: PreparedBatchResources,
    backend: BackendKind,
    outcome: PhantomData<fn() -> (R, E)>,
}

pub(super) fn prepare_map<'authority, T, K, R, E>(
    map: ExecutionMap<T, K>,
    lease: ExecutionResourceLease<'authority>,
) -> Result<PreparedExecutionMap<'authority, T, K, R, E>, LeaseDenial>
where
    T: Sync + ChargedBytes,
    R: Send + ChargedBytes,
    E: Send + ChargedBytes,
{
    let backend = if lease.resolved_posture() == ExecutionPosture::Automatic {
        BackendKind::Native
    } else {
        BackendKind::Serial
    };
    let resources = prepare_batch_resources::<T, R, E>(&lease, &map.batch, backend)?;
    Ok(PreparedExecutionMap {
        map,
        lease,
        resources,
        backend,
        outcome: PhantomData,
    })
}

impl<T, K, R, E> PreparedExecutionMap<'_, T, K, R, E>
where
    T: Sync + ChargedBytes,
    R: Send + ChargedBytes,
    E: Send + ChargedBytes,
{
    pub fn partition_count(&self) -> usize {
        self.map.partition_count()
    }

    /// Recheck the bound request context, acquire a worker, and consume the
    /// same memory ticket. The kernel receives fresh work/deadline limits.
    pub fn run<F>(self, kernel: F) -> MapOutcome<R, E>
    where
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        let outcome = run_checked_batch_prepared(
            &self.lease,
            &self.map.batch,
            self.backend,
            self.resources,
            &kernel,
        );
        outcome.into()
    }
}
