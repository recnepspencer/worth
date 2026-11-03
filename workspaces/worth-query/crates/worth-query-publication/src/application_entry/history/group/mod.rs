//! A typed owning root retains the original document and every ordinary group owner.
use super::*;
use crate::application_entry::{
    operation::{
        HistoryMode as Mode, HistoryOperationAdvance, HistoryOperationDriver,
        HistoryRecordingSource, WorthQueryHistoryOperationMode,
    },
    WorthQueryOperationDescription, WorthQuerySourceOperationBinding,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, NoApplicationMutationSource,
    },
    application_program::{ApplicationAction, ApplicationProgramDefinition},
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

type Operation<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::Operation;
type Completed<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::Completed;
type Terminal<S, I, P, A> = <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::Terminal;
type Closing<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::GroupClosing;
type Completion<S, I, P, A> =
    <Mode<S, I, A> as WorthQueryHistoryOperationMode<S, I, P, A>>::GroupCompletion;
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
    MembershipFrozen,
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

mod member;
use member::Member;
mod storage;
pub use storage::{
    WorthQueryHistoryGestureCompletion, WorthQueryHistoryGestureIntent,
    WorthQueryHistoryGestureMembers, WorthQueryHistoryGroupMembers, WorthQueryHistoryTypedMembers,
};
mod gesture;

/// A transient split borrow leaves every real owner inside this root.
#[must_use = "complete or abort the group and discharge its real owners explicitly"]
pub struct WorthQueryGroupedHistoryDocument<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    execution: WorthQueryHistoryGroupExecution<'runtime, S, H, P, M>,
}
#[must_use = "retain this execution inside its grouped document until cleanup finishes"]
pub struct WorthQueryHistoryGroupExecution<
    'runtime, S:ApplicationSchema,H:ApplicationHistoryBinding<S>,
    P:ApplicationProgramDefinition<S>,M:WorthQueryHistoryGroupMembers<S,H,P>,
> {
    program:WorthQueryProgramApplicationRuntime<S,P>,
    key:u128,
    handoff:Option<(WorthQueryHistoryGroupHandoffPointForTest,std::sync::Arc<super::owner_handoff::OwnerHandoff>)>,
    document_identity:worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryIdentity,
    members:M,
    reservation:Option<WorthQueryDocumentHistoryGroupReservation>,
    candidate:Option<WorthQueryExpectedProductBranch<'runtime,S>>,
    recovery:Option<super::edit::ForkRecovery>,
    cleanup:Option<super::edit::DiscardedCandidate<'runtime,S>>,
    aborting:Option<worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryGroupAborting>,
    selected:Option<WorthQueryHistoryStep>,
    next:usize,
    closing:usize,
    frozen:bool,
    posture:WorthQueryHistoryGroupPosture,
    _memory:Vec<worth_execution::ExecutionMemoryReservation>,
    marker:std::marker::PhantomData<fn()->H>,
}

impl<'runtime, S, H, P, M, Payload, Outputs>
    WorthQueryGroupedHistoryDocument<'runtime, S, H, P, M, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    pub fn parts(
        &mut self,
    ) -> (
        &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        &mut WorthQueryHistoryGroupExecution<'runtime, S, H, P, M>,
    ) {
        (&self.document, &mut self.execution)
    }
    pub fn document(
        &self,
    ) -> &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs> {
        &self.document
    }
    pub fn posture(&self) -> WorthQueryHistoryGroupPosture {
        self.execution.posture
    }
    pub fn completed_members(&self) -> usize {
        self.execution.completed_members()
    }
}

impl<S, H, P, M> WorthQueryHistoryGroupExecution<'_, S, H, P, M>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
{
    fn validate_document<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &self,
        document: &WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>,
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
        self.next
    }
    pub fn selected_step(&self) -> Option<&WorthQueryHistoryStep> {
        self.selected.as_ref()
    }
}
impl<S, H, P, M, Payload, Outputs> std::fmt::Debug
    for WorthQueryGroupedHistoryDocument<'_, S, H, P, M, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryGroupedHistoryDocument")
            .field("posture", &self.posture())
            .finish_non_exhaustive()
    }
}
