//! Request-funded capture of the original branch schema roster.
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};
use worth_relational_branch::internal_port::branch::transition::original_inputs::{
    OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation, RetainedBranchTransition,
};

/// An independently admitted capture roster. Cloning existing schema/record
/// Arcs does not copy their backing or retroactively fund ordinary producers.
#[derive(Debug)]
pub struct OriginalBranchSchemaInputs {
    pub birth: Option<OriginalEmptySchemaObservation>,
    pub installation: Option<OriginalEmptySchemaInstallation>,
    pub accepted_initial_inputs:
        Option<Arc<crate::durability::source_initial_inputs::image::InitialInputsImage>>,
    transitions: Vec<RetainedBranchTransition>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl OriginalBranchSchemaInputs {
    pub fn transitions(&self) -> &[RetainedBranchTransition] {
        &self.transitions
    }
}

#[derive(Debug)]
pub enum OriginalBranchSchemaInputsDenial {
    SizeOverflow,
    Request(WorkCeilingDenial),
    OriginalPrefixMismatch,
}

pub fn roster_bytes(capacity: usize) -> Option<u64> {
    let layout = std::alloc::Layout::array::<RetainedBranchTransition>(capacity).ok()?;
    u64::try_from(layout.size()).ok()
}

pub fn capture_producer_roster(
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
