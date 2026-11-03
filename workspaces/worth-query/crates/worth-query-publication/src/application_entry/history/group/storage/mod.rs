//! Member storage varies; the reservation, candidate and journal lifecycle do not.
use super::*;
pub(super) use worth_query_execution::publication_boundary::document_history::{
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryGroupAborting,
    WorthQueryDocumentHistoryOperationConstraint, WorthQueryHistoryGroupMemberAdmission,
};
pub(super) use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

mod heterogeneous;
mod typed;
pub(super) use heterogeneous::GestureInput;
pub use heterogeneous::{
    WorthQueryHistoryGestureCompletion, WorthQueryHistoryGestureIntent,
    WorthQueryHistoryGestureMembers,
};
pub use typed::WorthQueryHistoryTypedMembers;

/// Sealed storage of original SDK operation owners for one gesture.
/// ```compile_fail,E0277
/// use worth_query_publication::facade::application_entry::WorthQueryHistoryGroupMembers;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema,
///     application_history::ApplicationHistoryBinding,application_program::ApplicationProgramDefinition};
/// struct ClaimedMembers;
/// impl<S,H,P> WorthQueryHistoryGroupMembers<S,H,P> for ClaimedMembers
/// where S:ApplicationSchema,H:ApplicationHistoryBinding<S>,P:ApplicationProgramDefinition<S> {}
/// ```
pub trait WorthQueryHistoryGroupMembers<S, H, P>: GroupMembersDriver<S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
}

pub trait GroupMembersDriver<S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
    type Completion;
    fn validate_outputs(
        &self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial>;
    fn len(&self) -> usize;
    fn member(&self, index: usize) -> &dyn GroupMemberDriver<S, H, P>;
    fn member_mut(&mut self, index: usize) -> &mut dyn GroupMemberDriver<S, H, P>;
    fn finish_member(
        &mut self,
        index: usize,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), crate::application_entry::WorthQueryOperationCloseDenial>;
    fn into_completions(self) -> Vec<Self::Completion>
    where
        Self: Sized;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MemberPosture {
    Description,
    Unbound,
    Operation,
    Completed,
    Terminal,
    Closing,
    Closed,
}

pub trait GroupMemberDriver<S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
    fn posture(&self) -> MemberPosture;
    fn validate_outputs(
        &self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial>;
    fn construct(
        &mut self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial>;
    fn admit(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        owner: &WorthQueryDocumentHistory<'_, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
    ) -> Result<WorthQueryHistoryGroupMemberAdmission, WorthQueryHistoryGroupDenial>;
    fn bind(
        &mut self,
        constraint: WorthQueryDocumentHistoryOperationConstraint,
    ) -> Result<(), WorthQueryHistoryGroupDenial>;
    fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial>;
    fn recording_source<'a>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<HistoryRecordingSource<'a>, WorthQueryHistoryGroupDenial>;
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
    >;
    fn dispose(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<(), crate::application_entry::WorthQueryOperationCloseDenial>;
    fn resume_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial>;
}
