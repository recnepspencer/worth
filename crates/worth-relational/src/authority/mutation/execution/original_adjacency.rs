//! Original mutation deltas retain only their actual growable endpoint backing.
use crate::authority::mutation::AdjacencyDelta;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::TransactionCommitError as Error;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Default)]
pub(super) struct OriginalAdjacency {
    deltas: Vec<AdjacencyDelta>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl OriginalAdjacency {
    pub(super) fn as_slice(&self) -> &[AdjacencyDelta] {
        &self.deltas
    }
    pub(super) fn push(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        delta: AdjacencyDelta,
    ) -> Result<(), Error> {
        crate::execution::check_request(request).map_err(|cause| Error::execution(cause.into()))?;
        if self.deltas.len() == self.deltas.capacity() {
            let old = self.deltas.capacity();
            let next = if old == 0 {
                1
            } else {
                old.checked_mul(2)
                    .ok_or_else(|| Error::execution(Cause::ChargedBytesOverflow.into()))?
            };
            let bytes = |count| {
                std::alloc::Layout::array::<AdjacencyDelta>(count)
                    .ok()
                    .and_then(|layout| u64::try_from(layout.size()).ok())
                    .ok_or_else(|| Error::execution(Cause::ChargedBytesOverflow.into()))
            };
            if old != 0 && next <= old {
                return Err(Error::execution(Cause::ChargedBytesOverflow.into()));
            }
            let retained = bytes(next)?;
            let peak = retained
                .checked_add(bytes(old)?)
                .ok_or_else(|| Error::execution(Cause::ChargedBytesOverflow.into()))?;
            if let Some(memory) = &mut self._memory {
                memory.resize(peak).map_err(|cause| {
                    Error::execution(
                        crate::execution::denial::lease_denial(
                            worth_execution::LeaseDenial::MemoryExhausted(cause),
                        )
                        .into(),
                    )
                })?;
            } else {
                self._memory = request
                    .reserve_retained_memory(peak)
                    .map_err(|cause| Error::execution(cause.into()))?;
            }
            self.deltas
                .try_reserve_exact(next - self.deltas.len())
                .map_err(|_| Error::execution(Cause::AllocationFailed.into()))?;
            if let Some(memory) = &mut self._memory {
                memory
                    .resize(bytes(self.deltas.capacity())?)
                    .expect("releasing admitted adjacency overlap cannot exhaust memory");
            }
        }
        self.deltas.push(delta);
        Ok(())
    }
}
