//! Abort discharges original source owners, then candidate custody, then reserved metadata.
use super::*;

fn close_original<S, I, P, A>(
    closing: Closing<S, I, P, A>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    aborting:Option<&worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryGroupAborting>,
) -> Result<
    (),
    (
        Member<S, I, P, A>,
        crate::application_entry::WorthQueryOperationCloseDenial,
    ),
>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    let current = WorthQueryApplicationRequest {
        application: request.application,
        principal: request.principal,
        scope: request.scope,
        branch: Mode::<S, I, A>::group_disposal_target(&closing),
    };
    Mode::<S, I, A>::close_group_disposal(closing, &current, aborting)
        .map(drop)
        .map_err(|(closing, cause)| (Member::ClosingOriginal(Some(closing)), cause))
}

impl<'runtime, S, H, I, P, A, Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>
    WorthQueryGroupedHistoryDocument<'runtime, S, H, I, P, A, Payload, Outputs>
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
    /// This never reverses a publication. Earlier staged effects are retired
    /// through their actual owners while the document's selected head is preserved.
    pub fn abort(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        (Self, WorthQueryHistoryGroupDenial),
    > {
        use WorthQueryHistoryGroupDenial as Denial;
        if self.execution.document_identity
            != self
                .document
                .owner
                .history_document_identity(&program_publication_access())
        {
            return Err((
                self,
                Denial::History(WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::ForeignRecording,
                )),
            ));
        }
        if !std::ptr::eq(self.document.owner.application(), request.application) {
            return Err((
                self,
                Denial::History(WorthQueryApplicationHistoryDenial::ForeignRuntime),
            ));
        }
        if matches!(
            self.execution.posture,
            WorthQueryHistoryGroupPosture::Selected
                | WorthQueryHistoryGroupPosture::AlreadyRecorded
        ) {
            return Err((self, Denial::NotReady));
        }
        self.execution.posture = WorthQueryHistoryGroupPosture::Aborting;
        if let Some(reservation) = self.execution.reservation.take() {
            match self
                .document
                .owner
                .begin_history_group_abort(&program_publication_access(), reservation)
            {
                Ok(aborting) => self.execution.aborting = Some(aborting),
                Err((reservation, cause)) => {
                    self.execution.reservation = Some(reservation);
                    return Err((
                        self,
                        Denial::History(WorthQueryApplicationHistoryDenial::Document(cause)),
                    ));
                }
            }
        }
        while self.execution.closing < self.execution.members.len() {
            let index = self.execution.closing;
            let member = std::mem::replace(&mut self.execution.members[index], Member::Closed);
            let result = match member {
                Member::Description(_) => Ok(()),
                Member::Unbound(Some(operation)) | Member::Operation(Some(operation)) => {
                    match Mode::<S, I, A>::begin_group_disposal(operation) {
                        Ok(closing) => {
                            close_original(closing, request, self.execution.aborting.as_ref())
                        }
                        Err((operation, cause)) => Err((Member::Operation(Some(operation)), cause)),
                    }
                }
                Member::ClosingOriginal(Some(closing)) => {
                    close_original(closing, request, self.execution.aborting.as_ref())
                }
                Member::Completed(Some(completed)) => {
                    let current = WorthQueryApplicationRequest {
                        application: request.application,
                        principal: request.principal,
                        scope: request.scope,
                        branch: Mode::<S, I, A>::completed_target(&completed),
                    };
                    Mode::<S, I, A>::discard_completed(completed, &current)
                        .map_err(|(completed, cause)| (Member::Completed(Some(completed)), cause))
                }
                Member::Terminal(Some(terminal)) => {
                    match Mode::<S, I, A>::begin_group_terminal_disposal(terminal) {
                        Ok(closing) => {
                            close_original(closing, request, self.execution.aborting.as_ref())
                        }
                        Err((terminal, cause)) => Err((Member::Terminal(Some(terminal)), cause)),
                    }
                }
                _ => unreachable!("every unclosed original owner is retained"),
            };
            if let Err((member, cause)) = result {
                self.execution.members[index] = member;
                return Err((self, Denial::Close(cause)));
            }
            self.execution.closing += 1;
        }
        if let Some(recovery) = self.execution.recovery.take() {
            if let Err(recovery) = recovery.release() {
                self.execution.recovery = Some(recovery);
                return Err((
                    self,
                    Denial::Member(WorthQueryHistoryEditDenial::ForkRecovery),
                ));
            }
        }
        if let Some(candidate) = self.execution.candidate.take() {
            self.execution.cleanup = Some(super::super::edit::DiscardedCandidate::Live {
                head: candidate.into_owned_head(),
                denial: None,
            });
        }
        if let Some(candidate) = self.execution.cleanup.take() {
            match super::super::edit::discard::close_candidate(&self.document, request, candidate) {
                Ok(_) => {}
                Err((candidate, cause)) => {
                    self.execution.cleanup = Some(candidate);
                    return Err((self, Denial::Member(cause)));
                }
            }
        }
        if let Some(aborting) = self.execution.aborting.take() {
            if let Err((aborting, cause)) = self
                .document
                .owner
                .cancel_history_group(&program_publication_access(), aborting)
            {
                self.execution.aborting = Some(aborting);
                return Err((
                    self,
                    Denial::History(WorthQueryApplicationHistoryDenial::Document(cause)),
                ));
            }
        }
        self.execution.posture = WorthQueryHistoryGroupPosture::Aborted;
        Ok(self.document)
    }
}
