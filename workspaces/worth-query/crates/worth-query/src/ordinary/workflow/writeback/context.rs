use crate::runtime::{
    WorthQueryOrdinaryAuthorityAdmission, WorthQueryOrdinaryAuthorityFamily,
    WorthQueryRuntimeError, WorthQueryWorkspace,
};

use super::WorthQueryWorkflowCounters;

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryWritebackContext {
    pub(crate) authority: WorthQueryOrdinaryAuthorityAdmission,
}

#[derive(Debug)]
pub struct WorthQueryWritebackContextStop {
    error: WorthQueryRuntimeError,
    counters: WorthQueryWorkflowCounters,
}

impl WorthQueryWritebackContextStop {
    pub fn error(&self) -> &WorthQueryRuntimeError {
        &self.error
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }

    pub fn next_action(&self) -> super::WorthQueryWritebackNextAction {
        super::WorthQueryWritebackNextAction::RebindCurrentAuthority
    }
}

pub fn writeback(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &WorthQueryWorkspace,
) -> Result<WorthQueryWritebackContext, WorthQueryWritebackContextStop> {
    let authority = workspace
        .capture_ordinary_writeback_authority(execution)
        .map_err(|error| WorthQueryWritebackContextStop {
            error,
            counters: WorthQueryWorkflowCounters::context_checked(),
        })?;
    debug_assert_eq!(
        authority.family(),
        WorthQueryOrdinaryAuthorityFamily::Writeback
    );
    Ok(WorthQueryWritebackContext { authority })
}
