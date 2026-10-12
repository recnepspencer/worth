use crate::ordinary::workflow;
use crate::runtime::{WorthQueryEffectPolicy, WorthQueryWorkspace};
use crate::session_label::WorthQuerySessionLabel;

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryReadOnlyPreviewContext {
    pub(crate) authority: crate::runtime::WorthQueryOrdinaryAuthorityAdmission,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryPromotionPreviewContext {
    pub(crate) authority: crate::runtime::WorthQueryOrdinaryAuthorityAdmission,
}

pub type WorthQueryPreviewContextStop = workflow::WorthQueryWorkflowContextStop;

pub fn read_only(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &WorthQueryWorkspace,
    label: WorthQuerySessionLabel,
) -> Result<WorthQueryReadOnlyPreviewContext, WorthQueryPreviewContextStop> {
    workspace
        .capture_ordinary_preview_authority(execution, label, WorthQueryEffectPolicy::DeriveOnly)
        .map(|authority| WorthQueryReadOnlyPreviewContext { authority })
        .map_err(workflow::WorthQueryWorkflowContextStop::from_runtime)
}

pub fn promotion(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &WorthQueryWorkspace,
    label: WorthQuerySessionLabel,
) -> Result<WorthQueryPromotionPreviewContext, WorthQueryPreviewContextStop> {
    workflow::preview(request_execution, workspace, label).map(|context| {
        WorthQueryPromotionPreviewContext {
            authority: context.authority,
        }
    })
}
