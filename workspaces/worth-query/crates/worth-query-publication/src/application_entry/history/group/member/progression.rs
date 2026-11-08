//! Business advancement and receipt capture use the member's original delivery mode.
use super::super::storage::*;
use super::*;
impl<S, I, P, A> Member<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub(super) fn bind(
        &mut self,
        constraint: WorthQueryDocumentHistoryOperationConstraint,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        let Self::Unbound(operation) = self else {
            unreachable!("initial SDK owner")
        };
        match Mode::<S, I, A>::bind_candidate(operation.take().expect("initial owner"), constraint)
        {
            Ok(next) => *self = Self::Operation(Some(next)),
            Err((original, cause)) => {
                *operation = Some(original);
                return Err(WorthQueryHistoryGroupDenial::History(
                    WorthQueryApplicationHistoryDenial::Document(cause),
                ));
            }
        }
        Ok(())
    }
    pub(super) fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        let Self::Operation(operation) = self else {
            unreachable!("running member")
        };
        match Mode::<S, I, A>::advance(operation.take().expect("original SDK owner"), request) {
            HistoryOperationAdvance::Progress(next) => *operation = Some(next),
            HistoryOperationAdvance::Refused {
                operation: next,
                denial,
            } => {
                *operation = Some(next);
                return Err(WorthQueryHistoryGroupDenial::Member(denial));
            }
            HistoryOperationAdvance::Completed(next) => *self = Self::Completed(Some(next)),
            HistoryOperationAdvance::Terminal(next) => *self = Self::Terminal(Some(next)),
        }
        Ok(())
    }
    pub(super) fn recording_source<'a>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<HistoryRecordingSource<'a>, WorthQueryHistoryGroupDenial> {
        let Self::Completed(Some(completed)) = self else {
            unreachable!("actual completion")
        };
        Mode::<S, I, A>::recording_source(completed, phase, request)
            .map_err(WorthQueryHistoryGroupDenial::Member)?
            .ok_or(WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::ReceiptMismatch,
                ),
            ))
    }
    pub(super) fn resume_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        let Self::Operation(Some(operation)) = self else {
            return Err(WorthQueryHistoryGroupDenial::NotReady);
        };
        if !Mode::<S, I, A>::can_resume_group_cleanup(operation) {
            return Err(WorthQueryHistoryGroupDenial::NotReady);
        }
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::target(operation),
        };
        self.advance(&current)
    }
}
