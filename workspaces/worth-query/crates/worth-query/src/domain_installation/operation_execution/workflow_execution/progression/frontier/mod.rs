//! One frontier owns preparation, pure computation, and canonical application.
use std::collections::HashMap;
use worth_proof::CanonicalUniqueVec;

use super::super::workflow_provider::computation::WorkflowStageComputationIdentity;
use super::super::{
    WorthQueryWorkflowStageComputationFailure, WorthQueryWorkflowStageComputed,
    WorthQueryWorkflowStageTask, WorthQueryWorkflowValue,
};

pub(in crate::domain_installation::operation_execution) mod application;
mod computation;
mod preparation;

pub(in crate::domain_installation::operation_execution) use application::CanonicalWorkflowStageResult;

pub(in crate::domain_installation::operation_execution) struct PreparedWorkflowFrontier {
    identity: String,
    owner: String,
    order: CanonicalUniqueVec<String>,
    members: HashMap<String, PreparedWorkflowStage>,
    compute: fn(WorthQueryWorkflowStageTask) -> WorthQueryWorkflowStageComputed,
}

struct PreparedWorkflowStage {
    input: WorthQueryWorkflowValue,
    preparation: WorkflowStagePreparation,
}

enum WorkflowStagePreparation {
    Ready(WorthQueryWorkflowStageTask),
    Failed(WorthQueryWorkflowStageComputationFailure),
    Denied(super::super::WorthQueryWorkflowAdvanceDenialKind),
    Unstarted,
}

pub(in crate::domain_installation::operation_execution) struct ComputedWorkflowFrontier {
    identity: String,
    owner: String,
    order: CanonicalUniqueVec<String>,
    members: HashMap<String, ComputedWorkflowStage>,
}

struct ComputedWorkflowStage {
    input: WorthQueryWorkflowValue,
    computation: WorkflowStageComputation,
}

enum WorkflowStageComputation {
    Ready(WorthQueryWorkflowStageComputed),
    Failed(WorthQueryWorkflowStageComputationFailure),
    Denied(super::super::WorthQueryWorkflowAdvanceDenialKind),
    Unstarted,
}

#[cfg(test)]
mod tests;
