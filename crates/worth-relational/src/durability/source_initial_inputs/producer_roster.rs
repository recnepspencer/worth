//! Request-funded capture of the original branch schema roster.
use crate::branch::transition::original_inputs::{
    OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation, RetainedBranchTransition,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};

/// An independently admitted capture roster. Cloning existing schema/record
/// Arcs does not copy their backing or retroactively fund ordinary producers.
#[derive(Debug)]
pub(crate) struct OriginalBranchSchemaInputs {
    pub(crate) birth: Option<OriginalEmptySchemaObservation>,
    pub(crate) installation: Option<OriginalEmptySchemaInstallation>,
    pub(crate) accepted_initial_inputs:
        Option<Arc<crate::durability::source_initial_inputs::image::InitialInputsImage>>,
    transitions: Vec<RetainedBranchTransition>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl OriginalBranchSchemaInputs {
    pub(crate) fn transitions(&self) -> &[RetainedBranchTransition] {
        &self.transitions
    }
}

#[derive(Debug)]
pub(crate) enum OriginalBranchSchemaInputsDenial {
    SizeOverflow,
    Request(WorkCeilingDenial),
    OriginalPrefixMismatch,
}

pub(crate) fn roster_bytes(capacity: usize) -> Option<u64> {
    let layout = std::alloc::Layout::array::<RetainedBranchTransition>(capacity).ok()?;
    u64::try_from(layout.size()).ok()
}

pub(crate) fn capture_producer_roster(
    birth: Option<&OriginalEmptySchemaObservation>,
    installation: Option<&OriginalEmptySchemaInstallation>,
    accepted_initial_inputs: Option<&Arc<super::image::InitialInputsImage>>,
    selected: &[RetainedBranchTransition],
    request: &ExecutionRequest<'_, '_>,
) -> Result<OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial> {
    let bytes =
        roster_bytes(selected.len()).ok_or(OriginalBranchSchemaInputsDenial::SizeOverflow)?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(OriginalBranchSchemaInputsDenial::Request)?;
    let mut transitions = Vec::with_capacity(selected.len());
    for entry in selected {
        request
            .consult()
            .map_err(OriginalBranchSchemaInputsDenial::Request)?;
        transitions.push(entry.clone());
    }
    Ok(OriginalBranchSchemaInputs {
        birth: birth.cloned(),
        installation: installation.cloned(),
        accepted_initial_inputs: accepted_initial_inputs.cloned(),
        transitions,
        _memory: memory,
    })
}
