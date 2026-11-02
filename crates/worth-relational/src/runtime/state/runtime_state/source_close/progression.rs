//! The irreversible cut precedes all physical retirement and forbids resume.
use super::*;
impl RelationalSourceClose {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<RelationalSourceCloseProgress, RelationalSourceCloseDenial> {
        use RelationalSourceCloseDenial as Denial;
        if self.state().source_identity
            != (source as *const dyn RelationalDurableColdSource).cast::<()>() as usize
        {
            return Err(Denial::Medium(
                RelationalMediumRetirementDenial::ForeignSource,
            ));
        }
        if matches!(self.state().stage, Stage::Complete) {
            return Ok(RelationalSourceCloseProgress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        let state = self.state_mut();
        match state.stage {
            Stage::Quiesce => {
                match state
                    .runtime
                    .try_prepare_stopped_close()
                    .map_err(|()| Denial::Busy)?
                {
                    Some(cause) => return Err(Denial::Native(cause)),
                    None => {}
                }
                if !state
                    .runtime
                    .visibility
                    .cache
                    .try_has_memory_states()
                    .map_err(|()| Denial::Busy)?
                    && state.runtime.history.has_detached_retention_obligations()
                {
                    return Err(Denial::Native(
                        RelationalRuntimeCloseDenial::ReadCustodyRetained,
                    ));
                }
                state
                    .runtime
                    .durability
                    .preflight_source_retirement(source, request)
                    .map_err(Denial::Medium)?;
                state
                    .runtime
                    .partitions
                    .preflight_source_retirement()
                    .map_err(|()| Denial::Busy)?;
                state.graph = Some(graph::NativeSourceGraphRetirement::new(
                    &state.runtime,
                    NativeSourcePass::Validate,
                ));
                state.stage = Stage::Validate;
            }
            Stage::Validate => {
                if state
                    .graph
                    .as_mut()
                    .expect("original preflight cursor")
                    .advance(&state.runtime, request, source)?
                {
                    state.graph = None;
                    state.stage = Stage::Cache;
                }
            }
            Stage::Seal => {
                state.medium = Some(
                    state
                        .runtime
                        .durability
                        .seal_source_retirement(source, request)
                        .map_err(Denial::Medium)?,
                );
                // No fallible operation or callback separates this cut and posture.
                state.graph = Some(graph::NativeSourceGraphRetirement::new(
                    &state.runtime,
                    NativeSourcePass::Retire,
                ));
                state.stage = Stage::Retire;
            }
            Stage::Cache => {
                match state
                    .runtime
                    .visibility
                    .cache
                    .try_retire_memory_state(state.cache)
                    .map_err(|()| Denial::Busy)?
                {
                    Some(true) => {
                        state.cache_retired = true;
                        state.cache += 1;
                    }
                    Some(false) => state.cache_retired = true,
                    None => {
                        // Original cache pins have retired through their owners.
                        // Any remaining detached custody is genuinely external;
                        // refuse BEFORE sealing or retiring physical sources.
                        if state.runtime.history.has_detached_retention_obligations() {
                            return Err(Denial::Native(
                                RelationalRuntimeCloseDenial::ReadCustodyRetained,
                            ));
                        }
                        state.stage = Stage::Seal;
                    }
                }
            }
            Stage::Retire => {
                if state
                    .graph
                    .as_mut()
                    .expect("original retirement cursor")
                    .advance(&state.runtime, request, source)?
                {
                    state.graph = None;
                    state.stage = Stage::Medium;
                }
            }
            Stage::Medium => {
                if state
                    .medium
                    .as_mut()
                    .expect("same sealed provider")
                    .advance(request, source)
                    .map_err(Denial::Medium)?
                    == crate::durability::medium::RelationalMediumRetirementProgress::Complete
                {
                    state.stage = Stage::Source;
                }
            }
            Stage::Source => {
                state
                    .runtime
                    .durability
                    .retire_source_edge(source, request)
                    .map_err(Denial::Medium)?;
                state.stage = Stage::Business;
            }
            Stage::Business => {
                if let Some(cause) = state
                    .runtime
                    .complete_stopped_close_business()
                    .map_err(|()| Denial::Busy)?
                {
                    return Err(Denial::Native(cause));
                }
                state.stage = Stage::Complete;
                return Ok(RelationalSourceCloseProgress::Complete);
            }
            Stage::Complete => return Ok(RelationalSourceCloseProgress::Complete),
        }
        Ok(RelationalSourceCloseProgress::Advanced)
    }
}
