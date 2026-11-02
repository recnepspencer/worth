//! The extracted original binding exposes only physical retirement.
use super::*;
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalMediumRetirementDenial as Denial,
    RelationalMediumRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;

pub(crate) struct SealedRelationalMediumBinding {
    original: RelationalDurableMediumBinding,
    provider_complete: bool,
    complete: bool,
}
impl RelationalDurableMediumBinding {
    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        if self
            .cold_source
            .as_ref()
            .is_none_or(|original| !std::ptr::addr_eq(original.as_ref(), source))
        {
            return Err(Denial::ForeignSource);
        }
        if self.unresolved.get().is_some() {
            return Err(Denial::UnresolvedPublication);
        }
        let state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        // This hook is a non-consuming, non-reentrant provider qualification.
        // The durability ownership gate was released before entering here.
        state
            .medium
            .as_ref()
            .ok_or(Denial::MediumMoved)?
            .preflight_source_retirement(source, request)
    }
    pub(crate) fn into_source_retirement(self) -> SealedRelationalMediumBinding {
        SealedRelationalMediumBinding {
            original: self,
            provider_complete: false,
            complete: false,
        }
    }
}
impl SealedRelationalMediumBinding {
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Progress, Denial> {
        if self.complete {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        if self
            .original
            .cold_source
            .as_ref()
            .is_none_or(|original| !std::ptr::addr_eq(original.as_ref(), source))
        {
            return Err(Denial::ForeignSource);
        }
        if !self.provider_complete {
            // Exclusive ownership, not a held mutex guard, lends the SAME Box.
            let state = self
                .original
                .state
                .get_mut()
                .unwrap_or_else(|error| error.into_inner());
            let progress = state
                .medium
                .as_mut()
                .expect("sealed original provider was preflighted")
                .advance_source_retirement(source, request)?;
            self.provider_complete = progress == Progress::Complete;
            return Ok(Progress::Advanced);
        }
        drop(self.original.cold_source.take());
        self.complete = true;
        Ok(Progress::Complete)
    }
}
