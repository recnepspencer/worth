//! Workflow occurrence ownership composes the native source and outbox owners.
use super::*;
use crate::application_entry::{
    WorthQueryApplicationRequest, WorthQueryWorkflowOperationBindingDenial,
};
use worth_query_declaration::facade::application_program::{
    ApplicationAction, ApplicationActionDelivery, ApplicationSourceCompletion,
    ApplicationWorkflowSpec,
};
use worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowOperation;
mod advance;
mod completion;
mod construction;
mod inputs;
mod inspection;
mod local_completion;
pub use inspection::{
    WorthQueryWorkflowSourceInspection, WorthQueryWorkflowSourceInspectionDenial,
    WorthQueryWorkflowSourcePosture,
};
mod recovery;
pub use inputs::{
    WorthQueryCompletedWorkflowSelection, WorthQuerySelectedWorkflowOperation,
    WorthQueryTerminalWorkflowSelection, WorthQueryWorkflowInputDenial, WorthQueryWorkflowInputEnd,
    WorthQueryWorkflowInputList, WorthQueryWorkflowInputNode, WorthQueryWorkflowOperationChoice,
    WorthQueryWorkflowOperationInputs,
};

pub use completion::{
    WorthQueryCompletedWorkflowOperation, WorthQueryTerminalWorkflowOperation,
    WorthQueryWorkflowAcceptance,
};
pub use construction::WorthQueryOwnedWorkflowBinding;

/// The original issued occurrence, immutable description and native source custody.
#[must_use = "retain the original workflow occurrence and native cleanup"]
pub struct WorthQueryWorkflowOperation<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    required: RequiredWorkflowOperation,
    // Set only by the genuine completed source step, never by pending delivery.
    source_completed: bool,
    recovery: completion::WorkflowDispatchRecovery,
    recovery_result:
        Option<worth_query_execution::facade::primary_graph::WorthQueryRecoverySafeRetryAdmission>,
    // Release all outer native custody before the source's authentic runtime.
    source: WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>,
}
/// A stopped advance returns original custody; effect completion is not workflow terminality.
pub enum WorthQueryWorkflowOperationAdvance<O, C, T> {
    Progress(O),
    Refused {
        operation: O,
        denial: WorthQueryWorkflowOperationDenial,
    },
    Completed(C),
    Terminal(T),
}
impl<O, C, T> WorthQueryWorkflowOperationAdvance<O, C, T> {
    /// Structural composition preserves refusal custody and the original native denial.
    pub fn map<O2, C2, T2>(
        self,
        operation: impl FnOnce(O) -> O2,
        completed: impl FnOnce(C) -> C2,
        terminal: impl FnOnce(T) -> T2,
    ) -> WorthQueryWorkflowOperationAdvance<O2, C2, T2> {
        match self {
            Self::Progress(owner) => WorthQueryWorkflowOperationAdvance::Progress(operation(owner)),
            Self::Refused {
                operation: owner,
                denial,
            } => WorthQueryWorkflowOperationAdvance::Refused {
                operation: operation(owner),
                denial,
            },
            Self::Completed(owner) => {
                WorthQueryWorkflowOperationAdvance::Completed(completed(owner))
            }
            Self::Terminal(owner) => WorthQueryWorkflowOperationAdvance::Terminal(terminal(owner)),
        }
    }
}

pub use recovery::WorthQueryWorkflowEffectRecoveryDenial;

/// Actual native source or outbox refusal; neither authorizes an alternate retry path.
#[derive(Debug)]
pub enum WorthQueryWorkflowOperationDenial {
    Source(WorthQuerySourceOnlyDenial),
    Recovery(WorthQueryWorkflowEffectRecoveryDenial),
}
