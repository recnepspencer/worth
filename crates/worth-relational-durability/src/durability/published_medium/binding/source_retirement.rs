//! The extracted original binding exposes only physical retirement.
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::source_retirement::{
    RelationalMediumRetirementDenial as Denial, RelationalMediumRetirementProgress as Progress,
};

pub struct SealedRelationalMediumBinding {
    original: RelationalDurableMediumBinding,
    provider_complete: bool,
    complete: bool,
}
impl RelationalDurableMediumBinding {
    pub fn preflight_source_retirement(
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
        // An immutable indeterminate outcome is retained evidence, not running
        // Native work. The stopped-business preflight fences accepted work;
        // the original physical owner must classify its unresolved effects.
        // Keep this binding and its outcome through that inspection posture.
        let state = worth_execution::try_retirement_guard(self.state.try_lock(), Denial::Busy)?;
        // This hook is a non-consuming, non-reentrant provider qualification.
        // The durability ownership gate was released before entering here.
        state
            .medium
            .as_ref()
            .ok_or(Denial::MediumMoved)?
            .preflight_source_retirement(source, request)
    }
    pub fn into_source_retirement(self) -> SealedRelationalMediumBinding {
        SealedRelationalMediumBinding {
            original: self,
            provider_complete: false,
            complete: false,
        }
    }
}
impl SealedRelationalMediumBinding {
    pub fn advance(
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
