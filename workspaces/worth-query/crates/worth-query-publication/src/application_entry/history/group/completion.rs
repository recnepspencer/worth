//! Selection, delivery and each original result close remain distinct owned transitions.
use super::*;

pub struct WorthQueryHistoryGroupFinish<
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
    pub document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    pub step: WorthQueryHistoryStep,
    pub kind: WorthQueryHistoryGroupCompletionKind,
    pub completions: WorthQueryHistoryGroupCompletions<Completion<S, I, P, A>>,
}

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
    pub fn complete<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        self.validate_document(document, request)?;
        if self.selected.is_some() {
            document.current_step(request).map_err(Denial::History)?;
            return Ok(self.selected.as_ref().unwrap().clone());
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
                let Member::Completed(Some(completed)) = self.members.last().unwrap() else {
                    unreachable!("all members completed")
                };
                let (prepared, selected) = Mode::<S, I, A>::prepare_completed(
                    completed, &phase, &current,
                )
                .map_err(|cause| {
                    Denial::History(WorthQueryApplicationHistoryDenial::Mutation(cause))
                })?;
                let installed = runtime
                    .installed_schema()
                    .installed_history::<H>()
                    .map_err(|cause| {
                        Denial::History(WorthQueryApplicationHistoryDenial::Installation(cause))
                    })?;
                let adopted = document.owner.adopt_history_group::<I::Binding>(
                    &phase,
                    &program_publication_access(),
                    &installed,
                    WorthQueryDocumentHistoryOperation::new(
                        &selected,
                        &prepared.admission,
                        prepared.idempotency,
                    ),
                    self.reservation.take().expect("reserved group"),
                    self.candidate.take().expect("actual candidate"),
                );
                match adopted {
                    Ok(step) => {
                        self.selected = Some(step);
                        self.posture = WorthQueryHistoryGroupPosture::Selected;
                        self.pause_at(WorthQueryHistoryGroupHandoffPointForTest::Adoption);
                    }
                    Err(failure) => {
                        self.reservation = Some(failure.reservation);
                        self.candidate = Some(failure.candidate);
                        return Err(Denial::History(
                            WorthQueryApplicationHistoryDenial::Document(failure.denial),
                        ));
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
    /// Discharge every actual source result before returning the ordinary document.
    pub fn finish(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryGroupFinish<'runtime, S, H, I, P, A, Payload, Outputs>,
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
            let member = std::mem::replace(&mut self.execution.members[index], Member::Closed);
            let result = match member {
                Member::Completed(Some(completed)) => {
                    let current = WorthQueryApplicationRequest {
                        application: request.application,
                        principal: request.principal,
                        scope: request.scope,
                        branch: Mode::<S, I, A>::completed_target(&completed),
                    };
                    match Mode::<S, I, A>::finish_group_member(completed, &current) {
                        Ok(completion) => {
                            self.execution.completed.push(completion);
                            Ok(())
                        }
                        Err((completed, cause)) => Err((Member::Completed(Some(completed)), cause)),
                    }
                }
                Member::Unbound(Some(operation)) => {
                    let current = WorthQueryApplicationRequest {
                        application: request.application,
                        principal: request.principal,
                        scope: request.scope,
                        branch: Mode::<S, I, A>::target(&operation),
                    };
                    Mode::<S, I, A>::close_operation(operation, &current)
                        .map(|_| ())
                        .map_err(|(operation, cause)| (Member::Unbound(Some(operation)), cause))
                }
                _ => unreachable!("selected or original replay owner"),
            };
            if let Err((member, cause)) = result {
                self.execution.members[index] = member;
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
                self.execution.completed,
                self.execution._memory,
            ),
        })
    }
}
