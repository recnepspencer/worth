//! Install validated source DATA into the existing stream of the isolated receiver.
use super::*;
use worth_execution::ExecutionRequest;

impl RelationalCanonicalPublicationRoutes {
    /// Restore actual Memory originals through this isolated checkpoint head.
    /// Branch cells are already restored; these entries supply historical replay.
    /// Cost: O(covered original transitions), with request-funded retained backing.
    pub(crate) fn restore_retained_transition_prefix(
        &self,
        original: &OriginalBranchSchemaInputs,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), OriginalBranchSchemaInputsDenial> {
        use OriginalBranchSchemaInputsDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let covered = {
            let state = self.branch_transitions.state();
            if !state.log.is_empty() || state.covered != state.head {
                return Err(Denial::OriginalPrefixMismatch);
            }
            state.head
        };
        let count = original
            .transitions()
            .partition_point(|entry| entry.positioned.sequence() <= covered);
        if count as u64 != covered.0 {
            return Err(Denial::OriginalPrefixMismatch);
        }
        let memory = request
            .reserve_retained_memory(
                super::schema_inputs::roster_bytes(count).ok_or(Denial::SizeOverflow)?,
            )
            .map_err(Denial::Request)?;
        let mut transitions = Vec::with_capacity(count);
        for (offset, entry) in original.transitions()[..count].iter().enumerate() {
            request.consult().map_err(Denial::Request)?;
            if entry.positioned.sequence().0 != offset as u64 + 1 {
                return Err(Denial::OriginalPrefixMismatch);
            }
            transitions.push(entry.clone());
        }
        let mut state = self.branch_transitions.state();
        if !state.log.is_empty() || state.head != covered || state.covered != covered {
            return Err(Denial::OriginalPrefixMismatch);
        }
        // Memory owns this complete original prefix, so there is no external gap.
        state.log = transitions;
        state._log_memory = memory;
        state.covered = RelationalBranchTransitionSequence(0);
        Ok(())
    }

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
