//! One ordinary operation retains its original typed state throughout the group.
use super::*;
mod admission;
mod disposal;
mod progression;

pub(super) enum Member<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
> where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    Description(Option<WorthQueryOperationDescription<S, I>>),
    Unbound(Option<Operation<S, I, P, A>>),
    Operation(Option<Operation<S, I, P, A>>),
    Completed(Option<Completed<S, I, P, A>>),
    Terminal(Option<Terminal<S, I, P, A>>),
    ClosingOriginal(Option<Closing<S, I, P, A>>),
    Closed,
}

use super::storage::*;
impl<S, H, I, P, A> GroupMemberDriver<S, H, P> for Member<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    fn posture(&self) -> MemberPosture {
        match self {
            Self::Description(_) => MemberPosture::Description,
            Self::Unbound(_) => MemberPosture::Unbound,
            Self::Operation(_) => MemberPosture::Operation,
            Self::Completed(_) => MemberPosture::Completed,
            Self::Terminal(_) => MemberPosture::Terminal,
            Self::ClosingOriginal(_) => MemberPosture::Closing,
            Self::Closed => MemberPosture::Closed,
        }
    }
    fn validate_outputs(
        &self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        Mode::<S, I, A>::validate_group_outputs(program).map_err(|cause| {
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                cause,
            ))
        })
    }
    fn construct(
        &mut self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        Member::construct(self, program)
    }
    fn admit(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        owner: &WorthQueryDocumentHistory<'_, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
    ) -> Result<WorthQueryHistoryGroupMemberAdmission, WorthQueryHistoryGroupDenial> {
        Member::admit(self, phase, request, owner, installed)
    }
    fn bind(
        &mut self,
        constraint: WorthQueryDocumentHistoryOperationConstraint,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        Member::bind(self, constraint)
    }
    fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        Member::advance(self, request)
    }
    fn recording_source<'a>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<HistoryRecordingSource<'a>, WorthQueryHistoryGroupDenial> {
        Member::recording_source(self, phase, request)
    }
    fn adopt<'r>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        owner: &WorthQueryDocumentHistory<'r, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        group: WorthQueryDocumentHistoryGroupReservation,
        candidate: WorthQueryExpectedProductBranch<'r, S>,
    ) -> Result<
        WorthQueryHistoryStep,
        (
            WorthQueryDocumentHistoryGroupReservation,
            WorthQueryExpectedProductBranch<'r, S>,
            WorthQueryHistoryGroupDenial,
        ),
    > {
        Member::adopt(self, phase, request, owner, installed, group, candidate)
    }
    fn dispose(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<(), crate::application_entry::WorthQueryOperationCloseDenial> {
        Member::dispose(self, request, aborting)
    }
    fn resume_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        Member::resume_cleanup(self, request)
    }
}
