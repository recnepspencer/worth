use crate::application_entry::WorthQueryApplicationRequestMutationDenial;
use worth_query_execution::publication_boundary::workflow_advance::WorkflowTransitionPreparationDenial;

/// The kind of a `WorthQueryWorkflowAdvancePreparationDenial`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowAdvancePreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    TransitionPreparation,
    Authentication,
    AwaitingActor,
    BackActionUnavailable,
}

/// Why a workflow advance did not prepare. `RuntimeMismatch` means the workflow belongs to
/// another runtime.
#[derive(Debug)]
pub enum WorthQueryWorkflowAdvancePreparationDenial {
    /// No fresh business Back requirement was produced; preparation changed nothing.
    BackActionUnavailable,
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    TransitionPreparation(WorkflowTransitionPreparationDenial),
    Authentication(
        worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial,
    ),
    /// Observation-time readiness; no transition attempt was prepared.
    AwaitingActor(
        worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowActor,
    ),
}

impl WorthQueryWorkflowAdvancePreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowAdvancePreparationDenialKind {
        match self {
            Self::BackActionUnavailable => {
                WorthQueryWorkflowAdvancePreparationDenialKind::BackActionUnavailable
            }
            Self::RuntimeMismatch => {
                WorthQueryWorkflowAdvancePreparationDenialKind::RuntimeMismatch
            }
            Self::RequestAdmission(_) => {
                WorthQueryWorkflowAdvancePreparationDenialKind::RequestAdmission
            }
            Self::TransitionPreparation(_) => {
                WorthQueryWorkflowAdvancePreparationDenialKind::TransitionPreparation
            }
            Self::Authentication(_) => {
                WorthQueryWorkflowAdvancePreparationDenialKind::Authentication
            }
            Self::AwaitingActor(_) => WorthQueryWorkflowAdvancePreparationDenialKind::AwaitingActor,
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowAdvancePreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow advance preparation denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryWorkflowAdvancePreparationDenial {}
