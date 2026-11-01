//! Actual change/event backing and old/new reallocation overlap.
use super::MutationOutcome;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::TransactionCommitError as Error;
use worth_execution::ExecutionRequest;
impl MutationOutcome {
    pub(super) fn overflow() -> Error {
        Error::execution(Cause::ChargedBytesOverflow.into())
    }
    fn bytes(changes: usize, events: usize) -> Result<u64, Error> {
        let changes = std::alloc::Layout::array::<super::super::RecordMutation>(changes)
            .map_err(|_| Self::overflow())?
            .size();
        let events = std::alloc::Layout::array::<super::super::MutationEvent>(events)
            .map_err(|_| Self::overflow())?
            .size();
        changes
            .checked_add(events)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(Self::overflow)
    }
    pub(super) fn reserve(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        changes: usize,
        events: usize,
    ) -> Result<(), Error> {
        crate::execution::check_request(execution)
            .map_err(|cause| Error::execution(cause.into()))?;
        let old_changes = self.changes.capacity();
        let old_events = self.events.capacity();
        if changes <= old_changes && events <= old_events {
            return Ok(());
        }
        let next_changes = if changes > old_changes {
            old_changes
                .checked_mul(2)
                .ok_or_else(Self::overflow)?
                .max(changes)
                .max(1)
        } else {
            old_changes
        };
        let next_events = if events > old_events {
            old_events
                .checked_mul(2)
                .ok_or_else(Self::overflow)?
                .max(events)
                .max(1)
        } else {
            old_events
        };
        let retained = Self::bytes(next_changes, next_events)?;
        // Each realloc may retain its old allocation until its new one exists.
        let overlap = Self::bytes(
            if changes > old_changes {
                old_changes
            } else {
                0
            },
            if events > old_events { old_events } else { 0 },
        )?;
        let peak = retained.checked_add(overlap).ok_or_else(Self::overflow)?;
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
            self._memory = execution
                .reserve_retained_memory(peak)
                .map_err(|cause| Error::execution(cause.into()))?;
        }
        self.changes
            .try_reserve_exact(
                next_changes
                    .checked_sub(self.changes.len())
                    .expect("admitted capacity covers current rows"),
            )
            .map_err(|_| Error::execution(Cause::AllocationFailed.into()))?;
        self.events
            .try_reserve_exact(
                next_events
                    .checked_sub(self.events.len())
                    .expect("admitted capacity covers current events"),
            )
            .map_err(|_| Error::execution(Cause::AllocationFailed.into()))?;
        let actual = Self::bytes(self.changes.capacity(), self.events.capacity())?;
        if let Some(memory) = &mut self._memory {
            memory
                .resize(actual)
                .expect("releasing admitted outcome overlap cannot exhaust memory");
        }
        Ok(())
    }
}
