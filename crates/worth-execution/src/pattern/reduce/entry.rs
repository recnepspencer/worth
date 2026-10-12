//! The reduce pattern's public entries: a run that reserves its own memory,
//! and one that holds the caller's: its inputs' before, its tree's after.

use worth_foundational::{ExecutionPosture, ExecutionReport};

use super::super::ExecutionMap;
use super::ReduceInputDenial;
use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease},
    backend::{BackendKind, KernelContext, KernelFailure},
    oracle::CanonicalBits,
    reduction::{ReductionMetrics, ReductionTree},
    report::ChargedBytes,
};

type Reduced<R, Combine, E> =
    Result<(ReductionTree<R, Combine>, ExecutionReport, ReductionMetrics), ReduceInputDenial<E>>;

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
    /// Map values stay bound to checked identities while one parent scope
    /// retains their memory through canonical tree construction. `max_value_bytes`
    /// bounds each reducer value's inline and owned bytes and encoded length;
    /// `reducer_storage_bytes` declares heap storage captured by `combine`.
    pub fn run_reduce<R, E, Kernel, Combine>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
    ) -> Reduced<R, Combine, E>
    where
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        self.run_reduce_from(
            lease,
            None,
            None,
            kernel,
            identity,
            combine,
            max_value_bytes,
            reducer_storage_bytes,
        )
    }

    /// [`Self::run_reduce`] on the caller's memory. The map's admission takes
    /// over `inputs` as [`Self::run_taking`] does: the caller's reservation
    /// for what the partitions hold becomes the map's own in one ledger step.
    /// The reduction's admission takes over `tree` the same way and leaves it
    /// holding what the completed tree keeps, and nothing when the run stops,
    /// so the tree is never unreserved between the run's hold and the
    /// caller's.
    pub fn run_reduce_holding<R, E, Kernel, Combine>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        inputs: Option<ExecutionMemoryReservation>,
        tree: Option<&mut ExecutionMemoryReservation>,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
    ) -> Reduced<R, Combine, E>
    where
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        self.run_reduce_from(
            lease,
            inputs,
            tree,
            kernel,
            identity,
            combine,
            max_value_bytes,
            reducer_storage_bytes,
        )
    }

    fn run_reduce_from<R, E, Kernel, Combine>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        inputs: Option<ExecutionMemoryReservation>,
        tree: Option<&mut ExecutionMemoryReservation>,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
    ) -> Reduced<R, Combine, E>
    where
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        let Some(lease) = lease else {
            return self.run_reduce_serial(
                kernel,
                identity,
                combine,
                max_value_bytes,
                reducer_storage_bytes,
                inputs,
                tree,
            );
        };
        let backend = if lease.resolved_posture() == ExecutionPosture::Automatic {
            BackendKind::Native
        } else {
            BackendKind::Serial
        };
        self.run_reduce_leased(
            lease,
            kernel,
            identity,
            combine,
            max_value_bytes,
            reducer_storage_bytes,
            backend,
            true,
            None,
            inputs,
            tree,
        )
    }
}
