//! Fresh-request dispatch whose actual settled results keep native custody.
use super::{backend_for, ExecutionMap, MapKernelContext, MapKernelFailure, MapOutcome};
use crate::{
    backend::{run_retained_batch, run_retained_inputs_batch, ResultStorageDenial},
    ChargedBytes, ExecutionAllocationDenial, ExecutionMemoryReservation, ExecutionRequest,
    ExecutionRetainedResult, ExecutionWorkCeiling, LeaseDenial, WorkCeilingDenial,
};

#[derive(Debug)]
pub enum RetainedMapDenial {
    Request(WorkCeilingDenial),
    Admission(LeaseDenial),
    Allocation(ExecutionAllocationDenial),
    RequiresExecution(worth_foundational::PartitionIdentity),
}

impl RetainedMapDenial {
    pub(crate) fn interruption(stop: crate::MapKernelStop) -> Self {
        Self::Request(WorkCeilingDenial::Stopped(crate::backend::request_stop(
            stop,
        )))
    }
    pub(crate) fn storage(cause: ResultStorageDenial) -> Self {
        match cause {
            ResultStorageDenial::Admission(cause) => Self::Admission(cause),
            ResultStorageDenial::Allocation(cause) => Self::Allocation(cause),
            ResultStorageDenial::RequiresExecution(identity) => Self::RequiresExecution(identity),
        }
    }
}

impl<T: ChargedBytes + Sync, K> ExecutionMap<T, K> {
    pub(crate) fn run_retained_with_backend_taking<R, E, F>(
        &self,
        lease: Option<&crate::ExecutionResourceLease<'_>>,
        backend: crate::backend::BackendKind,
        inputs: Option<ExecutionMemoryReservation>,
        kernel: &F,
    ) -> Result<crate::backend::BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>
    where
        R: ChargedBytes + Send + Sync,
        E: ChargedBytes + Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        run_retained_batch(lease, &self.batch, backend, inputs, kernel)
    }
    /// Evaluate through the ordinary native backend, retaining each accepted
    /// result independently. A stopped suffix issues no result handle.
    ///
    /// The request is fresh and borrowed only for this call. Result handles
    /// carry no request, input equality, session, edition or model authority.
    /// Admission remains full for this arbitrary checked map. This method is
    /// not a sparse-dispatch API.
    pub fn run_retained<R, E, F>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        kernel: F,
    ) -> Result<MapOutcome<ExecutionRetainedResult<R>, E>, RetainedMapDenial>
    where
        R: ChargedBytes + Send + Sync,
        E: ChargedBytes + Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        let outcome = request
            .run(ceiling, |lease| {
                run_retained_batch(lease, &self.batch, backend_for(lease), None, &kernel)
            })
            .map_err(RetainedMapDenial::Request)?
            .0;
        request
            .check_live()
            .map_err(RetainedMapDenial::interruption)?;
        Ok(outcome.map_err(RetainedMapDenial::storage)?.into())
    }
}

impl<T: ChargedBytes + Sync, R: ChargedBytes + Send + Sync, K>
    ExecutionMap<super::RetainedMapInput<T, R>, K>
{
    /// Dispatch an explicit current map through the native owner. Carried
    /// inputs execute their real native checkpoint prefix without copying R.
    /// Their declared result capacity must cover the held handle's charged
    /// bytes; Evaluate capacities cover R as in the ordinary map. Unqualified
    /// or differently keyed carried results refuse before any kernel effect.
    pub fn run_retained_inputs<E, F>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        inputs: Option<ExecutionMemoryReservation>,
        kernel: F,
    ) -> Result<MapOutcome<ExecutionRetainedResult<R>, E>, RetainedMapDenial>
    where
        E: ChargedBytes + Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        let outcome = request
            .run(ceiling, |lease| {
                run_retained_inputs_batch(lease, &self.batch, backend_for(lease), inputs, &kernel)
            })
            .map_err(RetainedMapDenial::Request)?
            .0;
        request
            .check_live()
            .map_err(RetainedMapDenial::interruption)?;
        Ok(outcome.map_err(RetainedMapDenial::storage)?.into())
    }
}
