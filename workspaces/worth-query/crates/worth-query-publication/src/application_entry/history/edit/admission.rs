//! Fresh document READ and actual source-side preparation precede any protected attempt lookup.
use super::*;
use worth_query_declaration::facade::application_operation::NoApplicationMutationSource;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryRecordPreparation,
};

impl<'runtime, S, H, I, P, A, Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>
    WorthQueryHistoryEdit<'runtime, S, H, I, P, A, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: crate::application_entry::WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    #[inline(never)]
    pub(super) fn advance_admission(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        self.advance_admission_with(request, Ungoverned)
    }

    #[inline(never)]
    pub(super) fn advance_admission_with(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        metadata: impl MetadataAuthority<S, H>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        let context = match DocumentRequestContext::bind(&self.document.owner, request) {
            Ok(context) => context,
            Err(denial) => {
                return WorthQueryHistoryEditAdvance::Refused {
                    edit: self,
                    denial: WorthQueryHistoryEditDenial::History(denial),
                }
            }
        };
        let current = context.request();
        let mut already = None;
        let result = context.advance(|phase| {
            let Phase::Admission(state) = self.phase.as_mut().expect("admission phase") else {
                unreachable!()
            };
            let (prepared, selected) = Mode::<S, I, A>::prepare(
                state.operation.as_ref().expect("original action"),
                phase,
                current,
            )
            .map_err(WorthQueryApplicationHistoryDenial::Mutation)?;
            let prepared_record = context.admit_metadata(
                phase,
                self.document.access.clone(),
                &self.document.payload,
                metadata,
                |_, installed, read| {
                    self.document.owner.reserve_history_record::<I::Binding>(
                        phase,
                        &program_publication_access(),
                        installed,
                        WorthQueryDocumentHistoryOperation::new(
                            &selected,
                            &prepared.admission,
                            prepared.idempotency,
                        ),
                        read,
                    )
                },
            )?;
            match prepared_record {
                WorthQueryDocumentHistoryRecordPreparation::Reserved(reserved) => {
                    state.reservation = Some(reserved)
                }
                WorthQueryDocumentHistoryRecordPreparation::AlreadyRecorded(step) => {
                    already = Some(step)
                }
            }
            Ok(())
        });
        if let Some(step) = already {
            let Some(Phase::Admission(mut state)) = self.phase.take() else {
                unreachable!()
            };
            self.phase = Some(Phase::AlreadyRecorded {
                operation: state.operation.take().expect("unexecuted action"),
                step,
            });
        } else if matches!(&self.phase, Some(Phase::Admission(state)) if state.reservation.is_some())
        {
            let Some(Phase::Admission(state)) = self.phase.take() else {
                unreachable!()
            };
            self.phase = Some(Phase::Forking(state));
        }
        self.after_opening(result.map_err(WorthQueryHistoryEditDenial::History))
    }

    #[inline(never)]
    pub(super) fn advance_fork(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        let Some(Phase::Forking(state)) = self.phase.as_mut() else {
            unreachable!()
        };
        let mut recovery = state.recovery.take();
        let runtime = self.document.owner.application();
        let document = &self.document;
        let edit_phase = &mut self.phase;
        let result = super::super::recording::fork_candidate(
            document,
            request,
            &mut recovery,
            |phase, recovery| {
                let current = WorthQueryApplicationRequest {
                    application: runtime,
                    principal: request.principal,
                    scope: request.scope,
                    branch: document.owner.branch(),
                };
                let Some(Phase::Forking(state)) = edit_phase.as_mut() else {
                    unreachable!()
                };
                let (prepared, selected) = Mode::<S, I, A>::prepare(
                    state.operation.as_ref().expect("original initial action"),
                    phase,
                    &current,
                )
                .map_err(|cause| {
                    WorthQueryHistoryEditDenial::History(
                        WorthQueryApplicationHistoryDenial::Mutation(cause),
                    )
                })?;
                let installed = runtime
                    .installed_schema()
                    .installed_history::<H>()
                    .map_err(|cause| {
                        WorthQueryHistoryEditDenial::History(
                            WorthQueryApplicationHistoryDenial::Installation(cause),
                        )
                    })?;
                document
                    .owner
                    .readmit_history_record::<I::Binding>(
                        phase,
                        &program_publication_access(),
                        &installed,
                        WorthQueryDocumentHistoryOperation::new(
                            &selected,
                            &prepared.admission,
                            prepared.idempotency,
                        ),
                        state.reservation.as_ref().expect("actual reserved storage"),
                    )
                    .map_err(|cause| {
                        WorthQueryHistoryEditDenial::History(
                            WorthQueryApplicationHistoryDenial::Document(cause),
                        )
                    })?;
                if state.candidate.is_none() {
                    match document.owner.create_history_candidate(
                        phase,
                        &program_publication_access(),
                        state.reservation.as_ref().unwrap(),
                    ) {
                        Ok(candidate) => state.candidate = Some(candidate),
                        Err(cause) => return Err(ForkRecovery::creation_refused(recovery, cause)),
                    }
                }
                Ok(())
            },
        );
        if let Some(Phase::Forking(state)) = self.phase.as_mut() {
            state.recovery = recovery;
        }
        if matches!(&self.phase, Some(Phase::Forking(state)) if state.candidate.is_some()) {
            let Some(Phase::Forking(mut state)) = self.phase.take() else {
                unreachable!()
            };
            let restriction = self.document.owner.history_operation_constraint(
                &program_publication_access(),
                state.reservation.as_ref().unwrap(),
                state.candidate.as_ref().unwrap(),
            );
            let restriction = match restriction {
                Ok(value) => value,
                Err(cause) => {
                    self.phase = Some(Phase::Forking(state));
                    return WorthQueryHistoryEditAdvance::Refused {
                        edit: self,
                        denial: WorthQueryHistoryEditDenial::History(
                            WorthQueryApplicationHistoryDenial::Document(cause),
                        ),
                    };
                }
            };
            match Mode::<S, I, A>::bind_candidate(state.operation.take().unwrap(), restriction) {
                Ok(operation) => {
                    self.phase = Some(Phase::Running {
                        operation,
                        reservation: state.reservation.take().unwrap(),
                        candidate: state.candidate.take().unwrap(),
                    })
                }
                Err((operation, cause)) => {
                    state.operation = Some(operation);
                    self.phase = Some(Phase::Forking(state));
                    return WorthQueryHistoryEditAdvance::Refused {
                        edit: self,
                        denial: WorthQueryHistoryEditDenial::History(
                            WorthQueryApplicationHistoryDenial::Document(cause),
                        ),
                    };
                }
            }
        }
        self.after_opening(result)
    }

    fn after_opening(
        self,
        result: Result<(), WorthQueryHistoryEditDenial>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        match result {
            Err(denial) => WorthQueryHistoryEditAdvance::Refused { edit: self, denial },
            Ok(())
                if matches!(
                    self.posture(),
                    WorthQueryHistoryEditPosture::AlreadyRecorded
                ) =>
            {
                WorthQueryHistoryEditAdvance::Ready(self)
            }
            Ok(()) => WorthQueryHistoryEditAdvance::Progress(self),
        }
    }
}
