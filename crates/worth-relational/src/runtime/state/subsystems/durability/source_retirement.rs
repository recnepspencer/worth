//! Authentic publication-binding seal under its existing ownership gate.
use super::*;
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalMediumRetirementDenial as Denial,
    SealedRelationalMediumBinding,
};
use worth_execution::ExecutionRequest;
impl DurabilitySubsystem {
    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        let binding = {
            let state = self.state.try_read().ok_or(Denial::Busy)?;
            if state.transition_history.is_some() {
                return Err(Denial::TransitionReaderAttached);
            }
            if state
                .cold_source
                .as_ref()
                .is_none_or(|original| !std::ptr::addr_eq(original.as_ref(), source))
            {
                return Err(Denial::ForeignSource);
            }
            state.medium.clone().ok_or(Denial::MissingBinding)?
        };
        binding.preflight_source_retirement(source, request)
    }
    pub(crate) fn seal_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<SealedRelationalMediumBinding, Denial> {
        self.preflight_source_retirement(source, request)?;
        let mut state = self.state.try_write().ok_or(Denial::Busy)?;
        let original = state.medium.take().ok_or(Denial::MissingBinding)?;
        match Arc::try_unwrap(original) {
            Ok(original) => {
                drop(state);
                Ok(original.into_source_retirement())
            }
            Err(original) => {
                // Reinstall the EXACT Arc before any possible serving resume.
                state.medium = Some(original);
                Err(Denial::SharedBinding)
            }
        }
    }
    pub(crate) fn retire_source_edge(
        &self,
        source: &dyn RelationalDurableColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        let original = {
            let mut state = self.state.try_write().ok_or(Denial::Busy)?;
            if state.medium.is_some() {
                return Err(Denial::SharedBinding);
            }
            if state.transition_history.is_some() {
                return Err(Denial::TransitionReaderAttached);
            }
            if state
                .cold_source
                .as_ref()
                .is_some_and(|original| !std::ptr::addr_eq(original.as_ref(), source))
            {
                return Err(Denial::ForeignSource);
            }
            state.cold_source.take()
        };
        // The actual separately borrowed section source retains its original Arc.
        drop(original);
        Ok(())
    }
}
