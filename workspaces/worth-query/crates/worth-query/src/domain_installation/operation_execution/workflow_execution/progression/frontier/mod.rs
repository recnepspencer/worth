//! Original frontier membership accompanies ranked compute and canonical apply.
use super::super::workflow_provider::computation::WorkflowStageComputationIdentity;
use super::super::{
    WorthQueryWorkflowAdvanceDenialKind, WorthQueryWorkflowStageComputationFailure,
    WorthQueryWorkflowStageComputed, WorthQueryWorkflowStageTask, WorthQueryWorkflowValue,
};
use std::collections::HashMap;
use worth_execution::MapKernelContext;
use worth_proof::CanonicalUniqueVec;

pub(in crate::domain_installation::operation_execution) mod application;
mod computation;
mod preparation;
mod stop_conversion;
pub(in crate::domain_installation::operation_execution) use application::CanonicalWorkflowStageResult;

pub(in crate::domain_installation::operation_execution) struct PreparedWorkflowFrontier {
    identity: String,
    owner: String,
    order: CanonicalUniqueVec<String>,
    members: HashMap<usize, PreparedWorkflowStage>,
    compute: fn(
        WorthQueryWorkflowStageTask,
        &mut MapKernelContext<'_, '_>,
    ) -> WorthQueryWorkflowStageComputed,
    shape_denial: Option<WorthQueryWorkflowAdvanceDenialKind>,
}
struct PreparedWorkflowStage {
    input: WorthQueryWorkflowValue,
    preparation: WorkflowStagePreparation,
}
enum WorkflowStagePreparation {
    Ready(WorthQueryWorkflowStageTask),
    Failed(WorthQueryWorkflowStageComputationFailure),
    Denied(WorthQueryWorkflowAdvanceDenialKind),
}
pub(in crate::domain_installation::operation_execution) struct ComputedWorkflowFrontier {
    identity: String,
    owner: String,
    order: CanonicalUniqueVec<String>,
    prefix: Vec<ComputedWorkflowStage>,
    stop: Option<WorkflowFrontierStop>,
    charged_work: u64,
}
struct ComputedWorkflowStage {
    rank: usize,
    input: WorthQueryWorkflowValue,
    computed: WorthQueryWorkflowStageComputed,
}
struct WorkflowFrontierStop {
    rank: Option<usize>,
    input: Option<WorthQueryWorkflowValue>,
    failure: WorkflowFrontierFailure,
}
enum WorkflowFrontierFailure {
    Domain(WorthQueryWorkflowStageComputed),
    Preparation(WorthQueryWorkflowStageComputationFailure),
    Denied(WorthQueryWorkflowAdvanceDenialKind),
}
#[cfg(test)]
mod tests;
