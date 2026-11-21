//! A checkpoint capture is one stage of its request; its shared helpers charge it.
//!
//! Helpers that a capture and a recovery both use take a meter. Inside the
//! checkpoint stage the meter admits memory, allocates arrays and hashes on the
//! stage's own lineage and opens no scope. Outside any stage the original
//! request meters each operation as before.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial, MapKernelContext,
};

pub(crate) trait CaptureMeter<'request, 'authority: 'request> {
    /// The genuine request this meter charges.
    fn request(&self) -> ExecutionRequest<'request, 'authority>;
    fn reserve(&mut self, bytes: u64) -> Result<Option<ExecutionMemoryReservation>, Cause>;
    /// A zero-work boundary: cancellation and the deadline, no charge.
    fn boundary(&mut self) -> Result<(), Cause>;
    fn array<T>(
        &mut self,
        count: usize,
    ) -> Result<ExecutionRequestArrayBuilder<'request, 'authority, T>, ExecutionRequestArrayDenial>;
    fn locator(&mut self, bytes: &[u8]) -> Result<RelationalDurableMemberLocator, Cause>;
}

impl<'request, 'authority: 'request> CaptureMeter<'request, 'authority>
    for ExecutionRequest<'request, 'authority>
{
    fn request(&self) -> ExecutionRequest<'request, 'authority> {
        *self
    }
    fn reserve(&mut self, bytes: u64) -> Result<Option<ExecutionMemoryReservation>, Cause> {
        self.reserve_retained_memory(bytes).map_err(Into::into)
    }
    fn boundary(&mut self) -> Result<(), Cause> {
        crate::execution::check_request(self)
    }
    fn array<T>(
        &mut self,
        count: usize,
    ) -> Result<ExecutionRequestArrayBuilder<'request, 'authority, T>, ExecutionRequestArrayDenial>
    {
        ExecutionRequestArrayBuilder::allocate(*self, count)
    }
    fn locator(&mut self, bytes: &[u8]) -> Result<RelationalDurableMemberLocator, Cause> {
        crate::durability::log::native_checkpoint_codec::member_frame::locator_for_request(
            self, bytes,
        )
    }
}

/// The checkpoint capture's own stage and the request it belongs to.
pub(crate) struct StageMeter<'stage, 'scope, 'lease, 'request, 'authority> {
    pub(crate) execution: ExecutionRequest<'request, 'authority>,
    pub(crate) stage: &'stage mut MapKernelContext<'scope, 'lease>,
}

impl<'request, 'authority: 'request> CaptureMeter<'request, 'authority>
    for StageMeter<'_, '_, '_, 'request, 'authority>
{
    fn request(&self) -> ExecutionRequest<'request, 'authority> {
        self.execution
    }
    fn reserve(&mut self, bytes: u64) -> Result<Option<ExecutionMemoryReservation>, Cause> {
        self.execution
            .reserve_retained_memory_in_stage(self.stage, bytes)
            .map_err(Into::into)
    }
    fn boundary(&mut self) -> Result<(), Cause> {
        crate::execution::request_stage::checkpoint(self.stage, 0).map_err(Cause::from)
    }
    fn array<T>(
        &mut self,
        count: usize,
    ) -> Result<ExecutionRequestArrayBuilder<'request, 'authority, T>, ExecutionRequestArrayDenial>
    {
        ExecutionRequestArrayBuilder::allocate_in_stage(self.execution, self.stage, count)
    }
    fn locator(&mut self, bytes: &[u8]) -> Result<RelationalDurableMemberLocator, Cause> {
        crate::durability::log::native_checkpoint_codec::member_frame::locator_in_stage(
            self.stage, bytes,
        )
    }
}
