//! Heap storage carried across the owned map's custody boundary.
use super::computation::{
    WorkflowStageComputationIdentity, WorthQueryWorkflowStageComputationFailure,
    WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageComputed,
    WorthQueryWorkflowStageTask,
};
use worth_execution::ChargedBytes;
use worth_query_installation::facade::WorthQueryOperationFailureClass;

impl ChargedBytes for WorkflowStageComputationIdentity {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { frontier, stage } = self;
        frontier
            .additional_charged_bytes()
            .saturating_add(stage.additional_charged_bytes())
    }
}
impl ChargedBytes for WorthQueryWorkflowStageComputePayload {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Empty | Self::Bool(_) | Self::I64(_) | Self::U64(_) => 0,
            Self::Text(value) => value.additional_charged_bytes(),
            Self::Bytes(value) => value.additional_charged_bytes(),
            Self::Words(value) => value.additional_charged_bytes(),
        }
    }
}
impl ChargedBytes for WorthQueryWorkflowStageTask {
    fn additional_charged_bytes(&self) -> u64 {
        let Self {
            declaration: _,
            identity,
            payload,
        } = self;
        identity
            .additional_charged_bytes()
            .saturating_add(payload.additional_charged_bytes())
    }
}
impl ChargedBytes for WorthQueryWorkflowStageComputed {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { identity, result } = self;
        identity
            .additional_charged_bytes()
            .saturating_add(match result {
                Ok(value) => value.additional_charged_bytes(),
                Err(failure) => failure.additional_charged_bytes(),
            })
    }
}
impl ChargedBytes for WorthQueryWorkflowStageComputationFailure {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { class, detail } = self;
        let class_bytes = match class {
            WorthQueryOperationFailureClass::Domain(name) => name.additional_charged_bytes(),
            WorthQueryOperationFailureClass::InvalidInput
            | WorthQueryOperationFailureClass::Unsupported
            | WorthQueryOperationFailureClass::Conflict
            | WorthQueryOperationFailureClass::Dependency
            | WorthQueryOperationFailureClass::Indeterminate => 0,
        };
        detail
            .additional_charged_bytes()
            .saturating_add(class_bytes)
    }
}
