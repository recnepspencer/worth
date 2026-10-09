use super::*;

/// The phase that prevented a workflow instance request from preparing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowInstancePreparationDenialKind {
    RuntimeMismatch,
    RequestAdmission,
    InstancePreparation,
}

/// Why an instance lifecycle request (a start, migration, fork continuation
/// or cancellation) did not prepare.
#[derive(Debug)]
pub enum WorthQueryWorkflowInstancePreparationDenial {
    RuntimeMismatch,
    RequestAdmission(WorthQueryApplicationRequestMutationDenial),
    InstancePreparation(WorkflowInstancePreparationDenial),
}

impl WorthQueryWorkflowInstancePreparationDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowInstancePreparationDenialKind {
        match self {
            Self::RuntimeMismatch => {
                WorthQueryWorkflowInstancePreparationDenialKind::RuntimeMismatch
            }
            Self::RequestAdmission(_) => {
                WorthQueryWorkflowInstancePreparationDenialKind::RequestAdmission
            }
            Self::InstancePreparation(_) => {
                WorthQueryWorkflowInstancePreparationDenialKind::InstancePreparation
            }
        }
    }
}

impl std::fmt::Display for WorthQueryWorkflowInstancePreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuntimeMismatch => {
                formatter.write_str("workflow runtime belongs to another application")
            }
            Self::RequestAdmission(denial) => write!(
                formatter,
                "workflow instance request was not admitted: {denial}"
            ),
            Self::InstancePreparation(denial) => write!(
                formatter,
                "workflow instance request did not prepare: {denial}"
            ),
        }
    }
}

impl std::error::Error for WorthQueryWorkflowInstancePreparationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RuntimeMismatch => None,
            Self::RequestAdmission(denial) => Some(denial),
            Self::InstancePreparation(denial) => Some(denial),
        }
    }
}
