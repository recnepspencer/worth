//! Before-durability admission of the existing stream's concrete Vec backing.

use worth_execution::ExecutionRequest;

use super::{
    schema_inputs::roster_bytes, BranchTransitionStream, OriginalBranchSchemaInputsDenial,
};

impl BranchTransitionStream {
    pub(super) fn prepare_append(
        &self,
        request: &ExecutionRequest<'_, '_>,
        positioned: &crate::branch::PositionedBranchTransition,
    ) -> Result<Option<usize>, OriginalBranchSchemaInputsDenial> {
        let mut state = self.state();
        request
            .consult()
            .map_err(OriginalBranchSchemaInputsDenial::Request)?;
        match state
            .log
            .binary_search_by_key(&positioned.sequence(), |entry| entry.positioned.sequence())
        {
            Ok(index) if state.log[index].positioned.as_ref() == positioned => {
                return Ok(Some(index));
            }
            Ok(_) => return Err(OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch),
            Err(index) if index < state.log.len() => {
                return Err(OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch);
            }
            Err(_) => {}
        }
        if state.log.len() < state.log.capacity() {
            return Ok(None);
        }
        let capacity = state
            .log
            .capacity()
            .max(2)
            .checked_mul(2)
            .ok_or(OriginalBranchSchemaInputsDenial::SizeOverflow)?;
        let bytes = roster_bytes(capacity).ok_or(OriginalBranchSchemaInputsDenial::SizeOverflow)?;
        // Admit the full new backing while the old backing/grant are alive.
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(OriginalBranchSchemaInputsDenial::Request)?;
        let mut replacement = Vec::with_capacity(capacity);
        replacement.append(&mut state.log);
        let old = std::mem::replace(&mut state.log, replacement);
        let old_memory = state._log_memory.take();
        state._log_memory = memory;
        drop(old);
        drop(old_memory);
        Ok(None)
    }
}
