//! Install validated source DATA into the existing stream of the isolated receiver.
use super::*;
use worth_execution::ExecutionRequest;

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn restore_source_schema_inputs(
        &self,
        birth: OriginalEmptySchemaObservation,
        installation: Option<OriginalEmptySchemaInstallation>,
        initial: Arc<crate::durability::source_initial_inputs::image::InitialInputsImage>,
        transitions: Vec<RetainedBranchTransition>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), OriginalBranchSchemaInputsDenial> {
        request
            .consult()
            .map_err(OriginalBranchSchemaInputsDenial::Request)?;
        let mut state = self.branch_transitions.state();
        if state.birth.is_some()
            || state.installation.is_some()
            || !state.log.is_empty()
            || transitions.len() as u64 != state.head.0
            || transitions
                .iter()
                .enumerate()
                .any(|(offset, entry)| entry.positioned.sequence().0 != offset as u64 + 1)
        {
            return Err(OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch);
        }
        state.birth = Some(birth);
        state.installation = installation;
        state.initial_inputs = Some(initial);
        state.log = transitions;
        state._log_memory = memory;
        Ok(())
    }
}
