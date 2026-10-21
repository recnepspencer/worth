//! An occupied native roster branch, not a consumer operation switch, accepts its owner.
use super::{WorthQueryCompletedWorkflowSelection, WorthQueryWorkflowInputList};
use crate::application_entry::{
    WorthQueryWorkflowAcceptance, WorthQueryWorkflowAdvanceRequest,
    WorthQueryWorkflowOperationOwnerAcceptanceDenial,
};
use worth_query_declaration::facade::{
    application_program::{ApplicationProgramDefinition, ApplicationWorkflowSpec},
    application_schema::ApplicationSchema,
};

impl<S, Spec, O, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<'_, '_, '_, S, Spec, O, Input, Scope>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    O: 'static,
    Input: Clone + Send + Sync + 'static,
{
    /// Accept exactly the selected completion through its original native owner.
    /// This neither advances a successor nor releases the owner's independent cleanup.
    /// Every refusal returns the same opaque completed selection for fresh-authority retry.
    pub fn accept_selected_operation<P, R>(
        self,
        operation: WorthQueryCompletedWorkflowSelection<S, P, R>,
    ) -> Result<
        WorthQueryWorkflowAcceptance<WorthQueryCompletedWorkflowSelection<S, P, R>>,
        (
            WorthQueryCompletedWorkflowSelection<S, P, R>,
            WorthQueryWorkflowOperationOwnerAcceptanceDenial,
        ),
    >
    where
        P: ApplicationProgramDefinition<S>,
        R: WorthQueryWorkflowInputList<S, P>,
    {
        match R::accept_completed(operation.operation, self) {
            Ok(accepted) => Ok(WorthQueryWorkflowAcceptance {
                progress: accepted.progress,
                operation: WorthQueryCompletedWorkflowSelection {
                    operation: accepted.operation,
                    marker: std::marker::PhantomData,
                },
            }),
            Err((operation, cause)) => Err((
                WorthQueryCompletedWorkflowSelection {
                    operation,
                    marker: std::marker::PhantomData,
                },
                cause,
            )),
        }
    }
}
