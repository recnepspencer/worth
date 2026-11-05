//! Business work stays on the original ordinary SDK owners of the one private candidate.
use super::*;

impl<'runtime, S, H, I, P, A> WorthQueryHistoryGroupExecution<'runtime, S, H, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
        ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn advance<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryGroupPosture, WorthQueryHistoryGroupDenial> {
        self.validate_document(document, request)?;
        match self.posture {
            WorthQueryHistoryGroupPosture::Constructing => self.construct()?,
            WorthQueryHistoryGroupPosture::Admitting => self.admit(document, request)?,
            WorthQueryHistoryGroupPosture::CreatingCandidate => self.fork(document, request)?,
            WorthQueryHistoryGroupPosture::Running => self.run_member(document, request)?,
            _ => {}
        }
        Ok(self.posture)
    }

    fn run_member<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        if matches!(&self.members[self.next], Member::Completed(_)) {
            return self.capture_member(document, request);
        }
        if let Member::Unbound(operation) = &mut self.members[self.next] {
            let restriction = document
                .owner
                .application()
                .with_application_advancement(request.scope, |phase| {
                    document
                        .read_in_advancement(&phase, request)
                        .map_err(Denial::History)?;
                    document
                        .owner
                        .history_group_operation_constraint(
                            &phase,
                            &program_publication_access(),
                            self.reservation.as_ref().expect("reserved group"),
                            self.candidate.as_ref().expect("private candidate"),
                            self.next,
                        )
                        .map_err(|cause| {
                            Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
                        })
                })
                .map_err(|cause| {
                    Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
                })??;
            match Mode::<S, I, A>::bind_candidate(
                operation.take().expect("initial SDK owner"),
                restriction,
            ) {
                Ok(operation) => self.members[self.next] = Member::Operation(Some(operation)),
                Err((original, cause)) => {
                    *operation = Some(original);
                    return Err(Denial::History(
                        WorthQueryApplicationHistoryDenial::Document(cause),
                    ));
                }
            }
            return Ok(());
        }
        let Member::Operation(operation) = &mut self.members[self.next] else {
            unreachable!("running member owns an operation")
        };
        let current = WorthQueryApplicationRequest {
            application: document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: self.candidate.as_ref().expect("private candidate").branch(),
        };
        match Mode::<S, I, A>::advance(operation.take().expect("original SDK operation"), &current)
        {
            HistoryOperationAdvance::Progress(next) => *operation = Some(next),
            HistoryOperationAdvance::Refused {
                operation: next,
                denial,
            } => {
                *operation = Some(next);
                return Err(Denial::Member(denial));
            }
            HistoryOperationAdvance::Completed(completed) => {
                self.members[self.next] = Member::Completed(Some(completed))
            }
            HistoryOperationAdvance::Terminal(terminal) => {
                self.members[self.next] = Member::Terminal(Some(terminal));
                self.posture = WorthQueryHistoryGroupPosture::Terminal;
            }
        }
        Ok(())
    }

    fn capture_member<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        document
            .owner
            .application()
            .with_application_advancement(request.scope, |phase| {
                document
                    .read_in_advancement(&phase, request)
                    .map_err(Denial::History)?;
                let Member::Completed(Some(completed)) = &self.members[self.next] else {
                    unreachable!("actual completion")
                };
                let current = WorthQueryApplicationRequest {
                    application: document.owner.application(),
                    principal: request.principal,
                    scope: request.scope,
                    branch: self.candidate.as_ref().unwrap().branch(),
                };
                let source = Mode::<S, I, A>::recording_source(completed, &phase, &current)
                    .map_err(Denial::Member)?
                    .ok_or(Denial::History(
                        WorthQueryApplicationHistoryDenial::Document(
                            WorthQueryDocumentHistoryDenial::ReceiptMismatch,
                        ),
                    ))?;
                let reservation = self.reservation.as_mut().expect("reserved group");
                let candidate = self.candidate.as_ref().unwrap();
                match source {
                    HistoryRecordingSource::Source(receipt) => {
                        document.owner.capture_history_group_member(
                            &phase,
                            &program_publication_access(),
                            reservation,
                            candidate,
                            receipt,
                        )
                    }
                    HistoryRecordingSource::RecordedSource(receipt) => {
                        document.owner.capture_history_group_original_member(
                            &phase,
                            &program_publication_access(),
                            reservation,
                            candidate,
                            receipt,
                        )
                    }
                    HistoryRecordingSource::RequiredOutputs { source, completed } => {
                        document.owner.capture_history_group_required_outputs(
                            &phase,
                            &program_publication_access(),
                            reservation,
                            candidate,
                            source,
                            &completed,
                        )
                    }
                }
                .map_err(|cause| {
                    Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
                })?;
                self.next += 1;
                if self.next == self.members.len() {
                    self.posture = WorthQueryHistoryGroupPosture::ReadyToComplete;
                }
                Ok(())
            })
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
            })?
    }

    /// Query the actual staged head under fresh ordinary READ. This creates no
    /// history node or permission and exposes no candidate mutation selector.
    pub fn read_staged<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<Query<S, H>, Rows<S, H>>,
        WorthQueryHistoryGroupDenial,
    > {
        self.validate_document(document, request)?;
        document
            .current_step(request)
            .map_err(WorthQueryHistoryGroupDenial::History)?;
        let candidate = self
            .candidate
            .as_ref()
            .ok_or(WorthQueryHistoryGroupDenial::NotReady)?;
        WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: candidate.branch(),
        }
        .query(document.access.clone())
        .execute()
        .map_err(|cause| {
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Query(cause))
        })
    }
}
