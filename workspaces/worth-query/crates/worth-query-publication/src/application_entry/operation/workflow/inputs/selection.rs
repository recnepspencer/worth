//! Structural wrappers preserve the actual native pending, completed and terminal owners.
mod acceptance;
use super::roster::WorthQueryWorkflowInputList;
use crate::application_entry::{
    WorthQueryApplicationRequest, WorthQueryClosedOperation, WorthQueryOperationCloseDenial,
    WorthQueryWorkflowOperationAdvance,
};
use worth_query_declaration::facade::{
    application_program::ApplicationProgramDefinition, application_schema::ApplicationSchema,
};
use worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowOperation;
/// The transferred input has one actual native operation owner, not cloneable retry authority.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::{
///     WorthQuerySelectedWorkflowOperation, WorthQueryWorkflowInputList,
/// };
/// use worth_query_declaration::facade::{
///     application_schema::ApplicationSchema, application_program::ApplicationProgramDefinition,
/// };
/// fn cannot_clone<S, P, R>(owner: WorthQuerySelectedWorkflowOperation<S, P, R>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>, R: WorthQueryWorkflowInputList<S, P> {
///     let _ = owner.required(); // Valid observation on the same bounded owner.
///     let _ = owner.clone();
/// }
/// ```
/// The valid counterpart borrows the requirement, then returns original custody.
/// ```
/// use worth_query_publication::facade::application_entry::{
///     WorthQuerySelectedWorkflowOperation, WorthQueryWorkflowInputList,
/// };
/// use worth_query_declaration::facade::{
///     application_schema::ApplicationSchema, application_program::ApplicationProgramDefinition,
/// };
/// fn retain<S, P, R>(owner: WorthQuerySelectedWorkflowOperation<S, P, R>) -> WorthQuerySelectedWorkflowOperation<S, P, R>
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>, R: WorthQueryWorkflowInputList<S, P> {
///     let _ = owner.required();
///     owner
/// }
/// ```
#[must_use = "retain native operation custody after input transfer"]
pub struct WorthQuerySelectedWorkflowOperation<
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: WorthQueryWorkflowInputList<S, P>,
> {
    operation: R::Operation,
    marker: std::marker::PhantomData<fn() -> (S, P)>,
}
/// The selected operation's genuine completion evidence and native cleanup owner.
/// This does not mean the workflow or a multi-operation command has completed.
#[must_use = "completion still retains native evidence and cleanup"]
pub struct WorthQueryCompletedWorkflowSelection<
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: WorthQueryWorkflowInputList<S, P>,
> {
    operation: R::Completed,
    marker: std::marker::PhantomData<fn() -> (S, P)>,
}
/// Actual stopped operation custody; it may be closed but has no advance entrance.
#[must_use = "terminal custody is not a successful effect"]
pub struct WorthQueryTerminalWorkflowSelection<
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: WorthQueryWorkflowInputList<S, P>,
> {
    operation: R::Terminal,
    marker: std::marker::PhantomData<fn() -> (S, P)>,
}
impl<
        S: ApplicationSchema,
        P: ApplicationProgramDefinition<S>,
        R: WorthQueryWorkflowInputList<S, P>,
    > WorthQuerySelectedWorkflowOperation<S, P, R>
{
    pub(super) fn new(operation: R::Operation) -> Self {
        Self {
            operation,
            marker: std::marker::PhantomData,
        }
    }
    pub fn required(&self) -> &RequiredWorkflowOperation {
        R::required(&self.operation)
    }
    pub fn into_operation(self) -> R::Operation {
        self.operation
    }
    pub fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryWorkflowOperationAdvance<
        Self,
        WorthQueryCompletedWorkflowSelection<S, P, R>,
        WorthQueryTerminalWorkflowSelection<S, P, R>,
    > {
        R::advance(self.operation, request).map(
            Self::new,
            |operation| WorthQueryCompletedWorkflowSelection {
                operation,
                marker: std::marker::PhantomData,
            },
            |operation| WorthQueryTerminalWorkflowSelection {
                operation,
                marker: std::marker::PhantomData,
            },
        )
    }
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        R::close(self.operation, request).map_err(|(owner, cause)| (Self::new(owner), cause))
    }
}
impl<
        S: ApplicationSchema,
        P: ApplicationProgramDefinition<S>,
        R: WorthQueryWorkflowInputList<S, P>,
    > WorthQueryCompletedWorkflowSelection<S, P, R>
{
    pub fn required(&self) -> &RequiredWorkflowOperation {
        R::completed_required(&self.operation)
    }
    pub fn into_operation(self) -> R::Completed {
        self.operation
    }
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        R::close_completed(self.operation, request).map_err(|(operation, cause)| {
            (
                Self {
                    operation,
                    marker: std::marker::PhantomData,
                },
                cause,
            )
        })
    }
}
impl<
        S: ApplicationSchema,
        P: ApplicationProgramDefinition<S>,
        R: WorthQueryWorkflowInputList<S, P>,
    > WorthQueryTerminalWorkflowSelection<S, P, R>
{
    pub fn into_operation(self) -> R::Terminal {
        self.operation
    }
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        R::close_terminal(self.operation, request).map_err(|(operation, cause)| {
            (
                Self {
                    operation,
                    marker: std::marker::PhantomData,
                },
                cause,
            )
        })
    }
}
