//! Selection, delivery and each original result close remain distinct owned transitions.
use super::*;

pub struct WorthQueryHistoryGroupFinish<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    pub document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    pub step: WorthQueryHistoryStep,
    pub kind: WorthQueryHistoryGroupCompletionKind,
    pub completions: WorthQueryHistoryGroupCompletions<M::Completion>,
}

impl<'runtime, S, H, P, M> WorthQueryHistoryGroupExecution<'runtime, S, H, P, M>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn complete<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        self.validate_document(document, request)?;
        if let Some(selected) = &self.selected {
            document.current_step(request).map_err(Denial::History)?;
            return Ok(selected.clone());
        }
        if self.posture != WorthQueryHistoryGroupPosture::ReadyToComplete {
            return Err(Denial::NotReady);
        }
        let runtime = document.owner.application();
        runtime
            .with_application_advancement(request.scope, |phase| {
                document
                    .read_in_advancement(&phase, request)
                    .map_err(Denial::History)?;
                document
                    .owner
                    .readmit_history_group(
                        &program_publication_access(),
                        self.reservation.as_ref().expect("reserved group"),
                    )
                    .map_err(|cause| {
                        Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
                    })?;
                let current = WorthQueryApplicationRequest {
                    application: runtime,
                    principal: request.principal,
                    scope: request.scope,
                    branch: document.owner.branch(),
                };
                let installed = runtime
                    .installed_schema()
                    .installed_history::<H>()
                    .map_err(|cause| {
                        Denial::History(WorthQueryApplicationHistoryDenial::Installation(cause))
                    })?;
                let adopted = self.members.member(self.members.len() - 1).adopt(
                    &phase,
                    &current,
                    &document.owner,
                    &installed,
                    self.reservation.take().expect("reserved group"),
                    self.candidate.take().expect("actual candidate"),
                );
                match adopted {
                    Ok(step) => {
                        self.selected = Some(step);
                        self.posture = WorthQueryHistoryGroupPosture::Selected;
                        self.pause_at(WorthQueryHistoryGroupHandoffPointForTest::Adoption);
                    }
                    Err((reservation, candidate, denial)) => {
                        self.reservation = Some(reservation);
                        self.candidate = Some(candidate);
                        return Err(denial);
                    }
                }
                document
                    .owner
                    .validate_record_delivery(&phase, &program_publication_access())
                    .map_err(|cause| {
                        Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
                    })
            })
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
            })??;
        Ok(self
            .selected
            .as_ref()
            .expect("selected before delivery")
            .clone())
    }
}

impl<'runtime, S, H, P, M, Payload, Outputs>
    WorthQueryGroupedHistoryDocument<'runtime, S, H, P, M, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Discharge every actual source result before returning the ordinary document.
    pub fn finish(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryGroupFinish<'runtime, S, H, P, M, Payload, Outputs>,
        (Self, WorthQueryHistoryGroupDenial),
    > {
        if self.execution.document_identity
            != self
                .document
                .owner
                .history_document_identity(&program_publication_access())
        {
            return Err((
                self,
                WorthQueryHistoryGroupDenial::History(
                    WorthQueryApplicationHistoryDenial::Document(
                        WorthQueryDocumentHistoryDenial::ForeignRecording,
                    ),
                ),
            ));
        }
        if !std::ptr::eq(self.document.owner.application(), request.application) {
            return Err((
                self,
                WorthQueryHistoryGroupDenial::History(
                    WorthQueryApplicationHistoryDenial::ForeignRuntime,
                ),
            ));
        }
        if !matches!(
            self.execution.posture,
            WorthQueryHistoryGroupPosture::Selected
                | WorthQueryHistoryGroupPosture::AlreadyRecorded
        ) {
            return Err((self, WorthQueryHistoryGroupDenial::NotReady));
        }
        while self.execution.closing < self.execution.members.len() {
            let index = self.execution.closing;
            if let Err(cause) = self.execution.members.finish_member(index, request) {
                return Err((self, WorthQueryHistoryGroupDenial::Close(cause)));
            }
            self.execution.closing += 1;
        }
        Ok(WorthQueryHistoryGroupFinish {
            document: self.document,
            step: self.execution.selected.take().unwrap(),
            kind: if self.execution.posture == WorthQueryHistoryGroupPosture::Selected {
                WorthQueryHistoryGroupCompletionKind::Recorded
            } else {
                WorthQueryHistoryGroupCompletionKind::AlreadyRecorded
            },
            completions: WorthQueryHistoryGroupCompletions::new(
                self.execution.members.into_completions(),
                self.execution._memory,
            ),
        })
    }
}
