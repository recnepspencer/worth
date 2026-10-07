use worth_query_installation::facade::WorthQueryOperationFailureClass;

/// Owned worker data. No variant carries runtime authority or an executable door.
#[derive(Debug, PartialEq)]
pub enum WorthQueryWorkflowStageComputePayload {
    Empty,
    Bool(bool),
    I64(i64),
    U64(u64),
    Text(String),
    Bytes(Vec<u8>),
    Words(Vec<u64>),
}

#[derive(Debug)]
pub struct WorthQueryWorkflowStageComputationFailure {
    pub(super) class: WorthQueryOperationFailureClass,
    pub(super) detail: String,
}

impl WorthQueryWorkflowStageComputationFailure {
    pub fn new(class: WorthQueryOperationFailureClass, detail: impl Into<String>) -> Self {
        Self {
            class,
            detail: detail.into(),
        }
    }

    pub(in crate::domain_installation::operation_execution) fn into_executor_failure(
        self,
    ) -> super::WorthQueryWorkflowStageExecutorFailure {
        super::WorthQueryWorkflowStageExecutorFailure::new(self.class, self.detail)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_installation::operation_execution) struct WorkflowStageComputationIdentity {
    pub(in crate::domain_installation::operation_execution) frontier: String,
    pub(in crate::domain_installation::operation_execution) stage: String,
}

/// A preparation-produced task, consumed exactly once by a static compute step.
///
/// Both phase values are owned and sendable:
/// ```
/// use worth_query::facade::domain::{WorthQueryWorkflowStageTask, WorthQueryWorkflowStageComputed};
/// fn sendable<T: Send + 'static>() {}
/// sendable::<WorthQueryWorkflowStageTask>();
/// sendable::<WorthQueryWorkflowStageComputed>();
/// ```
/// An inert compute step consumes that same task:
/// ```
/// use worth_query::facade::domain::{WorthQueryWorkflowStageTask, WorthQueryWorkflowStageComputed};
/// fn compute(task: WorthQueryWorkflowStageTask) -> WorthQueryWorkflowStageComputed {
///     task.pass_through()
/// }
/// ```
/// Registered computation is a bare function pointer, never an owner-capturing
/// closure. These twins differ only in the value referenced by the closure.
/// ```
/// use worth_query::facade::domain::*;
/// fn registered(workspace: &mut WorthQueryWorkflowStageWorkspace<'_>) -> fn(
///     WorthQueryWorkflowStageTask, &mut worth_execution::MapKernelContext<'_, '_>
/// ) -> WorthQueryWorkflowStageComputed {
///     |task, _meter| { let _ = &task; task.pass_through() }
/// }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::*;
/// fn registered(workspace: &mut WorthQueryWorkflowStageWorkspace<'_>) -> fn(
///     WorthQueryWorkflowStageTask, &mut worth_execution::MapKernelContext<'_, '_>
/// ) -> WorthQueryWorkflowStageComputed {
///     |task, _meter| { let _ = &workspace; task.pass_through() }
/// }
/// ```
/// ```
/// use worth_query::facade::domain::*;
/// fn registered(reader: &mut WorthQueryStageArtifactReader<'_>) -> fn(
///     WorthQueryWorkflowStageTask, &mut worth_execution::MapKernelContext<'_, '_>
/// ) -> WorthQueryWorkflowStageComputed {
///     |task, _meter| { let _ = &task; task.pass_through() }
/// }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::*;
/// fn registered(reader: &mut WorthQueryStageArtifactReader<'_>) -> fn(
///     WorthQueryWorkflowStageTask, &mut worth_execution::MapKernelContext<'_, '_>
/// ) -> WorthQueryWorkflowStageComputed {
///     |task, _meter| { let _ = &reader; task.pass_through() }
/// }
/// ```
/// A task has no workspace or reader door:
/// ```
/// use worth_query::facade::domain::WorthQueryWorkflowStageTask;
/// fn compute(task: WorthQueryWorkflowStageTask) { let _ = task.payload(); }
/// ```
/// ```compile_fail
/// use worth_query::facade::domain::WorthQueryWorkflowStageTask;
/// fn compute(task: WorthQueryWorkflowStageTask) { let _ = task.workspace(); }
/// ```
#[derive(Debug)]
pub struct WorthQueryWorkflowStageTask {
    pub(super) declaration: super::computation_declaration::WorkflowStageComputationDeclaration,
    pub(super) identity: WorkflowStageComputationIdentity,
    pub(super) payload: WorthQueryWorkflowStageComputePayload,
}

impl WorthQueryWorkflowStageTask {
    pub(in crate::domain_installation::operation_execution) fn new(
        identity: WorkflowStageComputationIdentity,
        payload: WorthQueryWorkflowStageComputePayload,
    ) -> Self {
        Self {
            identity,
            payload,
            declaration: Default::default(),
        }
    }

    pub fn stage_identity(&self) -> &str {
        &self.identity.stage
    }

    pub fn payload(&self) -> &WorthQueryWorkflowStageComputePayload {
        &self.payload
    }

    pub fn complete(
        self,
        result: Result<
            WorthQueryWorkflowStageComputePayload,
            WorthQueryWorkflowStageComputationFailure,
        >,
    ) -> WorthQueryWorkflowStageComputed {
        WorthQueryWorkflowStageComputed::new(self.identity, result)
    }

    pub fn pass_through(self) -> WorthQueryWorkflowStageComputed {
        WorthQueryWorkflowStageComputed::new(self.identity, Ok(self.payload))
    }

    pub(in crate::domain_installation::operation_execution) fn identity(
        &self,
    ) -> &WorkflowStageComputationIdentity {
        &self.identity
    }
}

/// Only consuming a prepared task can make a computed result for that occurrence.
#[derive(Debug)]
pub struct WorthQueryWorkflowStageComputed {
    pub(super) identity: WorkflowStageComputationIdentity,
    pub(super) result:
        Result<WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageComputationFailure>,
}

impl WorthQueryWorkflowStageComputed {
    fn new(
        identity: WorkflowStageComputationIdentity,
        result: Result<
            WorthQueryWorkflowStageComputePayload,
            WorthQueryWorkflowStageComputationFailure,
        >,
    ) -> Self {
        Self { identity, result }
    }
    pub(in crate::domain_installation::operation_execution) fn failed(&self) -> bool {
        self.result.is_err()
    }

    pub(in crate::domain_installation::operation_execution) fn into_result(
        self,
        expected: &WorkflowStageComputationIdentity,
    ) -> Result<WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageComputationFailure>
    {
        if self.identity != *expected {
            return Err(WorthQueryWorkflowStageComputationFailure::new(
                WorthQueryOperationFailureClass::Indeterminate,
                "computed workflow result belongs to another stage occurrence",
            ));
        }
        self.result
    }
}
