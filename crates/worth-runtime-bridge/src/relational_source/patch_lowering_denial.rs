//! Typed refusal while lowering an owner-issued patch.
use super::RelationalBridgePublicationDenial;
use crate::facade::RelationalBridgeSourceError;
use worth_execution::WorkCeilingDenial;

pub(super) enum PatchLoweringDenial {
    Publication(RelationalBridgePublicationDenial),
    Execution(WorkCeilingDenial),
}
impl From<WorkCeilingDenial> for PatchLoweringDenial {
    fn from(value: WorkCeilingDenial) -> Self {
        Self::Execution(value)
    }
}
impl From<RelationalBridgePublicationDenial> for PatchLoweringDenial {
    fn from(value: RelationalBridgePublicationDenial) -> Self {
        Self::Publication(value)
    }
}
impl PatchLoweringDenial {
    pub(super) fn into_outcome(
        self,
    ) -> Result<super::RelationalBridgePublicationOutcome, RelationalBridgeSourceError> {
        match self {
            Self::Publication(denial) => Ok(worth_proof::TransitionOutcome::Denied(denial)),
            Self::Execution(denial) => {
                Err(RelationalBridgeSourceError::execution_denied(denial.into()))
            }
        }
    }
}
