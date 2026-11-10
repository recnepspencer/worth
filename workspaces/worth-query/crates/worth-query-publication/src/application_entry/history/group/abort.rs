//! Abort discharges original source owners, then candidate custody, then reserved metadata.
use super::*;

impl<'runtime, S, H, P, M, Payload, Outputs>
    WorthQueryGroupedHistoryDocument<'runtime, S, H, P, M, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
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
        if let Some(candidate) = &self.execution.candidate
            && let Err(cause) = self
                .document
                .outputs
                .discard_recording(request, candidate.branch())
        {
            return Err((self, Denial::Outputs(cause)));
        }
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
            if let Err(cause) = self
                .execution
                .members
                .member_mut(index)
                .dispose(request, self.execution.aborting.as_ref())
            {
                return Err((self, Denial::Close(cause)));
            }
            self.execution.closing += 1;
        }
        if let Some(recovery) = self.execution.recovery.take()
            && let Err(recovery) = recovery.release()
        {
            self.execution.recovery = Some(recovery);
            return Err((
                self,
                Denial::Member(WorthQueryHistoryEditDenial::ForkRecovery),
            ));
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
        if let Some(aborting) = self.execution.aborting.take()
            && let Err((aborting, cause)) = self
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
        self.execution.posture = WorthQueryHistoryGroupPosture::Aborted;
        Ok(self.document)
    }
}
