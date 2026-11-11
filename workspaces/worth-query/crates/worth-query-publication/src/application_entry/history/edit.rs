//! One move-only edit owns the document, private candidate and original SDK continuation.
use super::*;
use crate::application_entry::operation::{
    HistoryMode as Mode, HistoryOperationAdvance, HistoryOperationDriver, HistoryRecordingSource,
    WorthQueryHistoryOperationMode,
};
use crate::application_entry::{WorthQueryOperationCloseDenial, WorthQuerySourceOnlyDenial};
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
    application_program::{
        ApplicationAction, ApplicationProgramDefinition, ApplicationSourceCompletion,
    },
};
use worth_query_execution::{
    facade::product::{
        WorthQueryApplicationProductBranchCleanupFailure, WorthQueryExpectedProductBranch,
        WorthQueryOwnedProductHead, WorthQueryProductBranchCreationRecovery,
        WorthQueryProductBranchOwnerCleanupFailure,
    },
    publication_boundary::document_history::{
        WorthQueryDocumentHistoryCandidateDenial, WorthQueryDocumentHistoryReservation,
    },
};

mod admission;
mod capture_pause;
mod creation_cleanup;
pub(super) mod discard;
mod finish;
mod fixed_outcome;
pub use fixed_outcome::WorthQueryFixedHistoryEditOutcome;
mod handoff;
mod metadata;
mod paging;
mod progression;
mod recording;
mod sealing;
pub use finish::{WorthQueryHistoryEditFinish, WorthQueryHistoryEditOutcome};
#[doc(hidden)]
pub use handoff::WorthQueryHistoryEditHandoffPauseForTest;

pub(super) type Operation<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::Operation;
pub(super) type Completed<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::Completed;
pub(super) type Terminal<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::Terminal;
pub(super) type ClosedTerminal<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::ClosedTerminal;
pub(super) type Outcome<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::Outcome;

#[derive(Debug)]
pub enum WorthQueryHistoryEditDenial {
    History(WorthQueryApplicationHistoryDenial),
    Source(WorthQuerySourceOnlyDenial),
    Fixed(crate::application_entry::WorthQueryOperationAdvanceDenial),
    Delivery(crate::application_entry::WorthQueryOperationDeliveryDenial),
    Outputs(WorthQueryHistoryOutputDenial),
    OutputQualification(worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial),
    Close(WorthQueryOperationCloseDenial),
    Candidate(WorthQueryDocumentHistoryCandidateDenial),
    ForkRecovery,
    CandidateCleanup,
    NotReady,
    AlreadySelected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryEditPosture {
    AwaitingAdmission,
    CreatingCandidate,
    Running,
    AwaitingSelection,
    Selected,
    AlreadyRecorded,
    Terminal,
    Discarding,
}

#[must_use = "retain the original edit and every native owner after refusal"]
pub enum WorthQueryHistoryEditAdvance<Edit> {
    Progress(Edit),
    Ready(Edit),
    Refused {
        edit: Edit,
        denial: WorthQueryHistoryEditDenial,
    },
}

pub(super) enum ForkRecovery {
    Creation(WorthQueryProductBranchCreationRecovery),
    Cleanup(WorthQueryProductBranchOwnerCleanupFailure),
}

pub(super) struct Forking<
    'runtime,
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
> where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    pub(super) operation: Option<Operation<S, I, P, A>>,
    pub(super) reservation: Option<WorthQueryDocumentHistoryReservation>,
    pub(super) candidate: Option<WorthQueryExpectedProductBranch<'runtime, S>>,
    pub(super) recovery: Option<ForkRecovery>,
}

pub(super) struct Selecting<
    'runtime,
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
> where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    pub(super) completed: Option<Completed<S, I, P, A>>,
    pub(super) reservation: Option<WorthQueryDocumentHistoryReservation>,
    pub(super) candidate: Option<WorthQueryExpectedProductBranch<'runtime, S>>,
    pub(super) selected: Option<WorthQueryHistoryStep>,
}

pub(super) enum DiscardedCandidate<'runtime, S> {
    Live {
        head: WorthQueryOwnedProductHead<'runtime, S>,
        denial: Option<
            worth_query_execution::facade::product::WorthQueryApplicationProductBranchCloseDenial,
        >,
    },
    Retired(WorthQueryApplicationProductBranchCleanupFailure),
    ClosedDirect {
        _receipt: worth_query_execution::facade::product::WorthQueryProductBranchCloseReceipt,
    },
    ClosedCleanup {
        _receipt:
            worth_query_execution::facade::product::WorthQueryProductBranchOwnerCleanupReceipt,
    },
}

pub(super) enum Phase<
    'runtime,
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
> where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    Admission(Forking<'runtime, S, I, P, A>),
    Forking(Forking<'runtime, S, I, P, A>),
    Running {
        operation: Operation<S, I, P, A>,
        reservation: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
    },
    Selecting(Selecting<'runtime, S, I, P, A>),
    Selected {
        completed: Completed<S, I, P, A>,
        step: WorthQueryHistoryStep,
    },
    AlreadyRecorded {
        operation: Operation<S, I, P, A>,
        step: WorthQueryHistoryStep,
    },
    Terminal {
        terminal: Terminal<S, I, P, A>,
        reservation: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
    },
    TerminalCleanup {
        terminal: ClosedTerminal<S, I, P, A>,
        reservation: WorthQueryDocumentHistoryReservation,
        candidate: DiscardedCandidate<'runtime, S>,
    },
    Discarding {
        candidate: DiscardedCandidate<'runtime, S>,
        reservation: WorthQueryDocumentHistoryReservation,
    },
}

#[must_use = "finish or explicitly discard the owned edit; publication and cleanup are not callbacks"]
pub struct WorthQueryHistoryEdit<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    pub(super) document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    pub(super) phase: Option<Phase<'runtime, S, I, P, A>>,
    handoff: Option<std::sync::Arc<handoff::OwnerHandoff>>,
}

impl<
        'runtime,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryHistoryEdit<'runtime, S, H, I, P, A, Payload, Outputs>
where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    pub(super) fn new(
        document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        operation: Operation<S, I, P, A>,
    ) -> Self {
        Self {
            document,
            phase: Some(Phase::Admission(Forking {
                operation: Some(operation),
                reservation: None,
                candidate: None,
                recovery: None,
            })),
            handoff: None,
        }
    }
    pub fn posture(&self) -> WorthQueryHistoryEditPosture {
        match self
            .phase
            .as_ref()
            .expect("the edit owns its current phase")
        {
            Phase::Admission(_) => WorthQueryHistoryEditPosture::AwaitingAdmission,
            Phase::Forking(_) => WorthQueryHistoryEditPosture::CreatingCandidate,
            Phase::Running { .. } => WorthQueryHistoryEditPosture::Running,
            Phase::Selecting(state) if state.selected.is_some() => {
                WorthQueryHistoryEditPosture::Selected
            }
            Phase::Selecting(_) => WorthQueryHistoryEditPosture::AwaitingSelection,
            Phase::Selected { .. } => WorthQueryHistoryEditPosture::Selected,
            Phase::AlreadyRecorded { .. } => WorthQueryHistoryEditPosture::AlreadyRecorded,
            Phase::Terminal { .. } => WorthQueryHistoryEditPosture::Terminal,
            Phase::TerminalCleanup { .. } => WorthQueryHistoryEditPosture::Terminal,
            Phase::Discarding { .. } => WorthQueryHistoryEditPosture::Discarding,
        }
    }
    pub fn selected_step(&self) -> Option<&WorthQueryHistoryStep> {
        match self.phase.as_ref()? {
            Phase::Selected { step, .. } | Phase::AlreadyRecorded { step, .. } => Some(step),
            Phase::Selecting(state) => state.selected.as_ref(),
            _ => None,
        }
    }
    pub fn completed_source(&self) -> Option<&Completed<S, I, P, A>> {
        match self.phase.as_ref()? {
            Phase::Selected { completed, .. } => Some(completed),
            Phase::Selecting(state) => state.completed.as_ref(),
            _ => None,
        }
    }
    pub fn terminal_source(&self) -> Option<&Terminal<S, I, P, A>> {
        match self.phase.as_ref()? {
            Phase::Terminal { terminal, .. } => Some(terminal),
            Phase::TerminalCleanup { terminal, .. } => Mode::<S, I, A>::retained_terminal(terminal),
            _ => None,
        }
    }
    pub fn source_operation(&self) -> Option<&Operation<S, I, P, A>> {
        match self.phase.as_ref()? {
            Phase::Admission(state) | Phase::Forking(state) => state.operation.as_ref(),
            Phase::Running { operation, .. } | Phase::AlreadyRecorded { operation, .. } => {
                Some(operation)
            }
            _ => None,
        }
    }
    /// Original closed terminal data while candidate retirement is still owned.
    pub fn closed_terminal(&self) -> Option<&ClosedTerminal<S, I, P, A>> {
        match self.phase.as_ref()? {
            Phase::TerminalCleanup { terminal, .. } => Some(terminal),
            _ => None,
        }
    }
    pub fn fork_recovery(&self) -> Option<&WorthQueryProductBranchCreationRecovery> {
        match self.phase.as_ref()? {
            Phase::Forking(Forking {
                recovery: Some(ForkRecovery::Creation(owner)),
                ..
            }) => Some(owner),
            _ => None,
        }
    }
    pub fn candidate_cleanup_failure(
        &self,
    ) -> Option<&WorthQueryApplicationProductBranchCleanupFailure> {
        match self.phase.as_ref()? {
            Phase::Discarding {
                candidate: DiscardedCandidate::Retired(failure),
                ..
            }
            | Phase::TerminalCleanup {
                candidate: DiscardedCandidate::Retired(failure),
                ..
            } => Some(failure),
            _ => None,
        }
    }
    pub fn candidate_close_denial(
        &self,
    ) -> Option<
        &worth_query_execution::facade::product::WorthQueryApplicationProductBranchCloseDenial,
    > {
        match self.phase.as_ref()? {
            Phase::Discarding {
                candidate: DiscardedCandidate::Live { denial, .. },
                ..
            }
            | Phase::TerminalCleanup {
                candidate: DiscardedCandidate::Live { denial, .. },
                ..
            } => denial.as_ref(),
            _ => None,
        }
    }
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > std::fmt::Debug for WorthQueryHistoryEdit<'_, S, H, I, P, A, Payload, Outputs>
where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryEdit")
            .field("posture", &self.posture())
            .finish_non_exhaustive()
    }
}
#[cfg(test)]
#[path = "edit_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
