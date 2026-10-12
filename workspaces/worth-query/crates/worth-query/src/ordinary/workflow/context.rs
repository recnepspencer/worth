use crate::runtime::{
    WorthQueryEffectPolicy, WorthQueryOrdinaryAuthorityAdmission,
    WorthQueryOrdinaryAuthorityFamily, WorthQueryRuntimeError, WorthQueryWorkspace,
};
use crate::session_label::WorthQuerySessionLabel;

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowContext {
    pub(crate) authority: WorthQueryOrdinaryAuthorityAdmission,
}

#[derive(Debug)]
pub struct WorthQueryWorkflowContextStop {
    error: WorthQueryRuntimeError,
}

impl WorthQueryWorkflowContextStop {
    pub fn error(&self) -> &WorthQueryRuntimeError {
        &self.error
    }

    pub fn next_action(&self) -> super::WorthQueryWorkflowNextAction {
        super::WorthQueryWorkflowNextAction::ProvideAuthority
    }

    pub(crate) fn from_runtime(error: WorthQueryRuntimeError) -> Self {
        Self { error }
    }
}

pub fn preview(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &WorthQueryWorkspace,
    label: WorthQuerySessionLabel,
) -> Result<WorthQueryWorkflowContext, WorthQueryWorkflowContextStop> {
    let authority = workspace
        .capture_ordinary_preview_authority(
            execution,
            label,
            WorthQueryEffectPolicy::SandboxedWriteIntent,
        )
        .map_err(|error| WorthQueryWorkflowContextStop { error })?;
    debug_assert_eq!(
        authority.family(),
        WorthQueryOrdinaryAuthorityFamily::PromotionPreview
    );
    Ok(WorthQueryWorkflowContext { authority })
}
