use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease},
    backend::{run_owned_batch_taking, BackendKind},
    report::ChargedBytes,
};

use super::{backend_for, ExecutionMap, MapKernelContext, MapKernelFailure, MapOutcome};

impl<T: Send + ChargedBytes, K> ExecutionMap<T, K> {
    /// Consume this map, moving each dispatched input into exactly one kernel
    /// call. Inputs skipped on any stop or admission refusal are dropped before
    /// this call returns. Workers join before return; the map retains no inputs.
    /// The kernel owns the disposition of inputs it receives. A single input
    /// destructor panic is contained as `Panic`, including skipped inputs and
    /// admission cleanup; cleanup destroys all remaining inputs independently.
    /// Admission cleanup reports the least panicking identity with no completed
    /// prefix. Existing checkpoint-stop precedence still applies after a kernel.
    /// An unrelated unwind preserves its original panic while cleanup continues.
    /// Caught panic payloads are disposed under a second panic boundary; a second
    /// payload is leaked if its predecessor's destructor panics. Only a double
    /// panic during unwinding aborts the process.
    ///
    /// Serial dispatch moves each item directly without an order allocation.
    /// Native dispatch takes one mutex handoff per item, outside the kernel.
    /// Perturbed native dispatch additionally retains one usize index per item.
    /// Borrowed serial/native dispatch retain their direct/atomic-index costs.
    ///
    /// Inputs and results need only `Send`. The shared kernel needs `Sync`;
    /// per-item mutable state travels in the input. Lease selection and all
    /// ordering, stop, charging and report laws are the same as [`Self::run`].
    pub fn run_owned<R, E, F>(
        self,
        lease: Option<&ExecutionResourceLease<'_>>,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_owned_with_backend_taking(lease, backend_for(lease), None, kernel)
    }

    /// [`Self::run_owned`] with atomic takeover of the caller's input hold,
    /// as in [`Self::run_taking`]. The reservation remains held through worker
    /// completion and destruction of undispatched inputs, including admission
    /// refusal. Returned results leave the run's admission at this boundary.
    pub fn run_owned_taking<R, E, F>(
        self,
        lease: Option<&ExecutionResourceLease<'_>>,
        inputs: ExecutionMemoryReservation,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_owned_with_backend_taking(lease, backend_for(lease), Some(inputs), kernel)
    }

    pub(crate) fn run_owned_with_backend_taking<R, E, F>(
        self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        inputs: Option<ExecutionMemoryReservation>,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        run_owned_batch_taking(lease, self.batch, backend, inputs, &kernel).into()
    }
}
