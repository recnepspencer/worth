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
use crate::durability::log::native_checkpoint_codec::member_frame;
use crate::durability::medium::RelationalDurableMemberLocator;
#[cfg(test)]
use worth_execution::ExecutionAllocationPolicy;
use worth_execution::{
    ExecutionByteBuffer, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayDenial,
    MapKernelContext,
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
    pub(crate) fn reserve_read(
        &mut self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, Denial> {
        match self {
            Self::Original { execution, stage } => execution
                .reserve_retained_memory_in_stage(stage, bytes)
                .map_err(|cause| Denial::Request(cause.into())),
            #[cfg(test)]
            Self::Policy(_) => Ok(None),
        }
    }
    pub(crate) fn locator(
        &mut self,
        bytes: &[u8],
    ) -> Result<RelationalDurableMemberLocator, Denial> {
        match self {
            Self::Original { stage, .. } => {
                member_frame::locator_in_stage(stage, bytes).map_err(Denial::Request)
            }
            #[cfg(test)]
            Self::Policy(_) => Ok(member_frame::locator(bytes)),
        }
    }
}

fn allocation(cause: ExecutionRequestArrayDenial) -> Denial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => Denial::Request(cause.into()),
        ExecutionRequestArrayDenial::Allocation(cause) => Denial::Allocation(cause),
    }
}
