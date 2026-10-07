use super::computation::{
    WorkflowStageComputationIdentity, WorthQueryWorkflowStageComputePayload,
    WorthQueryWorkflowStageTask,
};
use super::WorthQueryWorkflowValue;

/// Input facts available before a frontier starts. Owner resources stay opaque.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorthQueryWorkflowStageInputFacts<'a> {
    NotRequired,
    Bool(bool),
    I64(i64),
    U64(u64),
    Text(&'a str),
    EntityIdentity(&'a str),
    OwnerOnly,
}

impl<'a> WorthQueryWorkflowStageInputFacts<'a> {
    pub(in crate::domain_installation::operation_execution) fn from_value(
        value: &'a WorthQueryWorkflowValue,
    ) -> Self {
        match value {
            WorthQueryWorkflowValue::NotRequired => Self::NotRequired,
            WorthQueryWorkflowValue::Bool(value) => Self::Bool(*value),
            WorthQueryWorkflowValue::I64(value) => Self::I64(*value),
            WorthQueryWorkflowValue::U64(value) => Self::U64(*value),
            WorthQueryWorkflowValue::Text(value) => Self::Text(value),
            WorthQueryWorkflowValue::EntityIdentity(value) => Self::EntityIdentity(value),
            WorthQueryWorkflowValue::CurrentEntityIdentity(_)
            | WorthQueryWorkflowValue::Projection(_)
            | WorthQueryWorkflowValue::InstalledArtifact(_)
            | WorthQueryWorkflowValue::TransferredArtifact(_) => Self::OwnerOnly,
        }
    }
}

pub struct WorthQueryWorkflowPreparationPredecessor<'a> {
    pub(in crate::domain_installation::operation_execution) identity: &'a str,
    pub(in crate::domain_installation::operation_execution) stage: &'a str,
    pub(in crate::domain_installation::operation_execution) output:
        WorthQueryWorkflowStageInputFacts<'a>,
}

impl<'a> WorthQueryWorkflowPreparationPredecessor<'a> {
    pub fn identity(&self) -> &str {
        self.identity
    }
    pub fn stage_identity(&self) -> &str {
        self.stage
    }
    pub fn output(&self) -> WorthQueryWorkflowStageInputFacts<'a> {
        self.output
    }
}

/// The preparation boundary carries fixed facts, never the execution context.
///
/// Reading inert input facts is valid:
/// ```
/// use worth_query::facade::domain::{WorthQueryWorkflowStagePreparation, WorthQueryWorkflowStageTask, WorthQueryWorkflowStageComputePayload};
/// fn prepare(view: WorthQueryWorkflowStagePreparation<'_>) -> WorthQueryWorkflowStageTask {
///     let _ = view.stage_identity();
///     let _ = view.input();
///     view.task(WorthQueryWorkflowStageComputePayload::Empty)
/// }
/// ```
/// Preparation has no execution-context door:
/// ```
/// use worth_query::facade::domain::WorthQueryWorkflowStagePreparation;
/// fn prepare(view: WorthQueryWorkflowStagePreparation<'_>) { let _ = view.input(); }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::WorthQueryWorkflowStagePreparation;
/// fn prepare(view: WorthQueryWorkflowStagePreparation<'_>) { let _ = view.execution_context(); }
/// ```
pub struct WorthQueryWorkflowStagePreparation<'a> {
    identity: WorkflowStageComputationIdentity,
    input: WorthQueryWorkflowStageInputFacts<'a>,
    predecessors: Vec<WorthQueryWorkflowPreparationPredecessor<'a>>,
}

impl<'a> WorthQueryWorkflowStagePreparation<'a> {
    pub(in crate::domain_installation::operation_execution) fn new(
        identity: WorkflowStageComputationIdentity,
        input: WorthQueryWorkflowStageInputFacts<'a>,
        predecessors: Vec<WorthQueryWorkflowPreparationPredecessor<'a>>,
    ) -> Self {
        Self {
            identity,
            input,
            predecessors,
        }
    }

    pub fn stage_identity(&self) -> &str {
        &self.identity.stage
    }
    pub fn input(&self) -> WorthQueryWorkflowStageInputFacts<'a> {
        self.input
    }
    pub fn predecessors(&self) -> &[WorthQueryWorkflowPreparationPredecessor<'a>] {
        &self.predecessors
    }

    pub fn task(
        self,
        payload: WorthQueryWorkflowStageComputePayload,
    ) -> WorthQueryWorkflowStageTask {
        WorthQueryWorkflowStageTask::new(self.identity, payload)
    }
}
