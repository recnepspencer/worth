//! One checkpoint capture is one stage of its genuine request.
//!
//! Work rate, charged cumulatively on that stage's meter against the request's
//! remaining work: one unit per byte each serializer pass produces (a sizing
//! pass and an emission pass are each charged), one per payload byte hashed and
//! one per byte copied. A zero-work boundary checkpoint charges nothing. Every
//! charge and boundary observes cancellation and the deadline; none opens a
//! scope, consults the request, or takes a reservation. Retained buffers admit
//! their exact bytes on the stage's memory lineage once, before allocation.
use worth_execution::{
    ExecutionByteBuffer, ExecutionRequest, ExecutionRequestArrayDenial, MapKernelContext,
};
use worth_relational_contracts::internal_port::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;

pub(crate) struct CheckpointEncodingControl<'stage, 'scope, 'lease, 'request, 'authority> {
    pub(crate) execution: ExecutionRequest<'request, 'authority>,
    pub(crate) stage: &'stage mut MapKernelContext<'scope, 'lease>,
}

impl CheckpointEncodingControl<'_, '_, '_, '_, '_> {
    pub(crate) fn checkpoint(&mut self, work: u64) -> Result<(), Denial> {
        worth_relational_contracts::internal_port::execution::request_stage::checkpoint(
            self.stage, work,
        )
        .map_err(|stop| Denial::Request(stop.into()))
    }

    pub(super) fn allocate(&mut self, bytes: usize) -> Result<ExecutionByteBuffer, Denial> {
        ExecutionByteBuffer::allocate_in_stage(bytes, self.execution, self.stage).map_err(allocation)
    }
}

fn allocation(cause: ExecutionRequestArrayDenial) -> Denial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => Denial::Request(cause.into()),
        ExecutionRequestArrayDenial::Allocation(cause) => Denial::Allocation(cause),
    }
}
