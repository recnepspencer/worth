//! One checkpoint capture is one stage of its genuine request; legacy policy is test-only.
//!
//! Work rate, charged cumulatively on that stage's meter against the request's
//! remaining work: one unit per byte each serializer pass produces (a sizing
//! pass and an emission pass are each charged), one per payload byte hashed and
//! one per byte copied. A zero-work boundary checkpoint charges nothing. Every
//! charge and boundary observes cancellation and the deadline; none opens a
//! scope, consults the request, or takes a reservation. Retained buffers admit
//! their exact bytes on the stage's memory lineage once, before allocation.
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
#[cfg(test)]
use worth_execution::ExecutionAllocationPolicy;
use worth_execution::{
    ExecutionByteBuffer, ExecutionRequest, ExecutionRequestArrayDenial, MapKernelContext,
};

pub(crate) enum CheckpointEncodingControl<'stage, 'scope, 'lease, 'request, 'authority> {
    Original {
        execution: ExecutionRequest<'request, 'authority>,
        stage: &'stage mut MapKernelContext<'scope, 'lease>,
    },
    #[cfg(test)]
    Policy(ExecutionAllocationPolicy<'request, 'authority>),
}

impl CheckpointEncodingControl<'_, '_, '_, '_, '_> {
    pub(crate) fn checkpoint(&mut self, work: u64) -> Result<(), Denial> {
        match self {
            Self::Original { stage, .. } => {
                crate::execution::request_stage::checkpoint(stage, work)
                    .map_err(|stop| Denial::Request(stop.into()))
            }
            #[cfg(test)]
            Self::Policy(policy) => policy.check_live().map_err(Denial::Allocation),
        }
    }
    pub(super) fn allocate(&mut self, bytes: usize) -> Result<ExecutionByteBuffer, Denial> {
        match self {
            Self::Original { execution, stage } => {
                ExecutionByteBuffer::allocate_in_stage(bytes, *execution, stage).map_err(allocation)
            }
            #[cfg(test)]
            Self::Policy(policy) => {
                ExecutionByteBuffer::allocate(bytes, *policy).map_err(Denial::Allocation)
            }
        }
    }
}

fn allocation(cause: ExecutionRequestArrayDenial) -> Denial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => Denial::Request(cause.into()),
        ExecutionRequestArrayDenial::Allocation(cause) => Denial::Allocation(cause),
    }
}
