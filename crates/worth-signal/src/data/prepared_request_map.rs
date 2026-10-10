//! Memory admitted before callbacks, with custody borrowed from their caller.
use crate::data::error::SignalError;
use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionMemoryReservation, ExecutionRequest, MapKernelContext,
    MapKernelFailure, MapOutcome, PreparedExecutionMap,
};
use worth_foundational::ExecutionPosture;

pub(crate) struct PreparedRequestMap<'request, 'authority, T, K, R, E> {
    _request: ExecutionRequest<'request, 'authority>,
    backing: PreparedMapBacking<'authority, T, K, R, E>,
}

enum PreparedMapBacking<'authority, T, K, R, E> {
    Leased(PreparedExecutionMap<'authority, T, K, R, E>),
    Serial {
        map: ExecutionMap<T, K>,
        memory: ExecutionMemoryReservation,
    },
}

impl<'request, 'authority, T, K, R, E> PreparedRequestMap<'request, 'authority, T, K, R, E>
where
    T: Sync + ChargedBytes,
    R: Send + ChargedBytes,
    E: Send + ChargedBytes,
{
    pub(crate) fn prepare(
        map: ExecutionMap<T, K>,
        request: ExecutionRequest<'request, 'authority>,
        posture: ExecutionPosture,
    ) -> Result<Self, SignalError> {
        let backing = request
            .in_scope(|lease| {
                if let Some(lease) = lease {
                    let child = lease
                        .child(worth_execution::LeaseRequest {
                            policy: worth_foundational::ExecutionRequestPolicy::new(
                                posture,
                                lease.policy().determinism(),
                                lease.policy().budget(),
                            ),
                            deadline: None,
                            cancellation: worth_execution::CancellationToken::new(),
                        })
                        .map_err(SignalError::execution_admission_denied)?;
                    map.prepare_run(child)
                        .map(PreparedMapBacking::Leased)
                        .map_err(SignalError::execution_admission_denied)
                } else {
                    let bytes = map.serial_memory_requirement::<R, E>().ok_or_else(|| {
                        SignalError::execution_admission_denied(
                            worth_execution::LeaseDenial::ChargedBytesOverflow,
                        )
                    })?;
                    let memory = ExecutionMemoryReservation::reserve_in_scope(None, bytes)
                        .map_err(SignalError::execution_admission_denied)?;
                    Ok(PreparedMapBacking::Serial { map, memory })
                }
            })
            .map_err(SignalError::execution_scope_denied)??;
        Ok(Self {
            _request: request,
            backing,
        })
    }

    pub(crate) fn partition_count(&self) -> usize {
        match &self.backing {
            PreparedMapBacking::Leased(map) => map.partition_count(),
            PreparedMapBacking::Serial { map, .. } => map.partition_count(),
        }
    }

    pub(crate) fn run<F>(self, kernel: F) -> MapOutcome<R, E>
    where
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        match self.backing {
            PreparedMapBacking::Leased(map) => map.run(kernel),
            PreparedMapBacking::Serial { map, memory } => map.run_taking(None, memory, kernel),
        }
    }
}

#[cfg(test)]
mod tests;
