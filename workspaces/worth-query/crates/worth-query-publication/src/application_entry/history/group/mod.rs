//! A typed owning root retains the original document and all group cleanup.
use super::*;
use crate::application_entry::{
    operation::{HistoryOperationAdvance, HistoryOperationDriver, HistoryRecordingSource},
    WorthQueryClosedOperation, WorthQueryCompletedSourceOperation, WorthQueryOperationDescription,
    WorthQuerySourceCompletion, WorthQuerySourceOnlyOperation, WorthQuerySourceOperationBinding,
    WorthQueryTerminalSourceOperation,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryModel,
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
        NoApplicationMutationSource,
    },
    application_program::{
        ApplicationAction, ApplicationActionDelivery, ApplicationProgramDefinition,
        ApplicationSourceCompletion as Mode,
    },
};
use worth_query_execution::{
    facade::{
        application_installation::WorthQueryProgramApplicationRuntime,
        product::WorthQueryExpectedProductBranch,
    },
    publication_boundary::document_history::{
        WorthQueryDocumentHistoryGroupPreparation, WorthQueryDocumentHistoryGroupReservation,
        WorthQueryDocumentHistoryOperation,
    },
};

mod admission;
mod cleanup;
mod completion;
mod construction;
mod handoff;
#[doc(hidden)]
pub use handoff::{
    WorthQueryHistoryGroupHandoffPauseForTest, WorthQueryHistoryGroupHandoffPointForTest,
};
mod progression;
mod results;
pub use completion::WorthQueryHistoryGroupFinish;
pub use results::{
    WorthQueryHistoryGroupCompletionIter, WorthQueryHistoryGroupCompletionKind,
    WorthQueryHistoryGroupCompletions,
};
mod abort;

type Operation<S, I, P, A> = WorthQuerySourceOnlyOperation<S, I, P, A, Mode>;
type Completed<S, I, P, A> = WorthQueryCompletedSourceOperation<S, I, P, A, Mode>;
type Terminal<S, I, P, A> = WorthQueryTerminalSourceOperation<S, I, P, A, Mode>;
type Key<S, I> =
    <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::IdempotencyKey;

#[derive(Debug)]
pub enum WorthQueryHistoryGroupDenial {
    History(WorthQueryApplicationHistoryDenial),
    Input(crate::application_entry::WorthQueryOperationInputCaptureDenial),
    Construction(crate::application_entry::WorthQueryOperationConstructionDenial),
    Member(WorthQueryHistoryEditDenial),
    Close(crate::application_entry::WorthQueryOperationCloseDenial),
    Memory(worth_execution::LeaseDenial),
    Work(worth_execution::WorkCeilingDenial),
    Capacity,
    NotReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryGroupPosture {
    Constructing,
    Admitting,
    CreatingCandidate,
    CandidateUnavailable,
    Running,
    ReadyToComplete,
    Selected,
    AlreadyRecorded,
    Terminal,
    Aborting,
    Aborted,
}

enum Member<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
> {
    Description(Option<WorthQueryOperationDescription<S, I>>),
    Unbound(Option<Operation<S, I, P, A>>),
    Operation(Option<Operation<S, I, P, A>>),
    Completed(Option<Completed<S, I, P, A>>),
    Terminal(Option<Terminal<S, I, P, A>>),
    ClosingOriginal(
        Option<
            crate::application_entry::operation::WorthQueryClosingSourceOperation<S, I, P, A, Mode>,
        >,
    ),
    Closed,
}

type Completion<S, I> = WorthQuerySourceCompletion<
    <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::Result,
>;

/// The transient split borrow cannot extract the original document or any live
/// SDK owner. Dropping that borrow leaves this root's actual cleanup intact.
#[must_use = "complete or abort the group and discharge its real owners explicitly"]
pub struct WorthQueryGroupedHistoryDocument<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
> {
    document: WorthQueryApplicationHistoryDocument<'runtime, S, H>,
    execution: WorthQueryHistoryGroupExecution<'runtime, S, H, I, P, A>,
}

#[must_use = "retain this execution inside its grouped document until cleanup finishes"]
pub struct WorthQueryHistoryGroupExecution<'runtime,S:ApplicationSchema,H:ApplicationHistoryBinding<S>,I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,A> {
    program: WorthQueryProgramApplicationRuntime<S,P>,
    key: u128,
    handoff:Option<(WorthQueryHistoryGroupHandoffPointForTest,std::sync::Arc<super::owner_handoff::OwnerHandoff>)>,
    document_identity: worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryIdentity,
    members: Vec<Member<S,I,P,A>>,
    completed: Vec<(WorthQueryClosedOperation,Completion<S,I>)>,
    reservation: Option<WorthQueryDocumentHistoryGroupReservation>,
    candidate: Option<WorthQueryExpectedProductBranch<'runtime,S>>,
    recovery: Option<super::edit::ForkRecovery>,
    cleanup: Option<super::edit::DiscardedCandidate<'runtime,S>>,
    aborting: Option<worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryGroupAborting>,
    selected: Option<WorthQueryHistoryStep>,
    next: usize,
    closing: usize,
    posture: WorthQueryHistoryGroupPosture,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    marker: std::marker::PhantomData<fn()->H>,
}

impl<
        'runtime,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
    > WorthQueryGroupedHistoryDocument<'runtime, S, H, I, P, A>
{
    pub fn parts(
        &mut self,
    ) -> (
        &WorthQueryApplicationHistoryDocument<'runtime, S, H>,
        &mut WorthQueryHistoryGroupExecution<'runtime, S, H, I, P, A>,
    ) {
        (&self.document, &mut self.execution)
    }
    pub fn document(&self) -> &WorthQueryApplicationHistoryDocument<'runtime, S, H> {
        &self.document
    }
    pub fn posture(&self) -> WorthQueryHistoryGroupPosture {
        self.execution.posture
    }
    pub fn completed_members(&self) -> usize {
        self.execution.completed_members()
    }
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
    > WorthQueryHistoryGroupExecution<'_, S, H, I, P, A>
{
    fn validate_document(
        &self,
        document: &WorthQueryApplicationHistoryDocument<'_, S, H>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        if !std::ptr::eq(self.program.runtime(), request.application)
            || !std::ptr::eq(self.program.runtime(), document.owner.application())
        {
            return Err(WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::ForeignRuntime,
            ));
        }
        if self.document_identity
            != document
                .owner
                .history_document_identity(&program_publication_access())
        {
            return Err(WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::ForeignRecording,
                ),
            ));
        }
        Ok(())
    }

    pub fn completed_members(&self) -> usize {
        self.reservation.as_ref().map_or_else(
            || {
                self.members
                    .iter()
                    .filter(|member| matches!(member, Member::Completed(_) | Member::Closed))
                    .count()
            },
            |reservation| reservation.completed_members(),
        )
    }
    pub fn selected_step(&self) -> Option<&WorthQueryHistoryStep> {
        self.selected.as_ref()
    }
    pub fn terminal_source(&self) -> Option<&Terminal<S, I, P, A>> {
        self.members.iter().find_map(|member| {
            if let Member::Terminal(terminal) = member {
                terminal.as_ref()
            } else {
                None
            }
        })
    }
    pub fn completed_source(&self, index: usize) -> Option<&Completed<S, I, P, A>> {
        match self.members.get(index)? {
            Member::Completed(completed) => completed.as_ref(),
            _ => None,
        }
    }
    pub fn source_operation(&self, index: usize) -> Option<&Operation<S, I, P, A>> {
        match self.members.get(index)? {
            Member::Unbound(operation) | Member::Operation(operation) => operation.as_ref(),
            _ => None,
        }
    }
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
    > std::fmt::Debug for WorthQueryGroupedHistoryDocument<'_, S, H, I, P, A>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryGroupedHistoryDocument")
            .field("posture", &self.posture())
            .finish_non_exhaustive()
    }
}
