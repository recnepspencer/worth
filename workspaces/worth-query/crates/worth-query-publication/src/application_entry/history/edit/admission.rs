//! Fresh document READ and actual source-side preparation precede any protected attempt lookup.
use super::*;
use crate::application_entry::operation::SourceAssessmentDriver;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryModel,
    application_operation::{ApplicationMutationScopeBinding, NoApplicationMutationSource},
};
use worth_query_execution::{
    facade::product::WorthQueryProductBranchCreateError,
    publication_boundary::document_history::{
        WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryRecordPreparation,
    },
};

impl<'runtime, S, H, I, P, A> WorthQueryHistoryEdit<'runtime, S, H, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    I::Binding: crate::application_entry::WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
        ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub(super) fn advance_admission(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        let runtime = self.document.owner.application();
        let mut already = None;
        let result = runtime.with_application_advancement(request.scope, |phase| {
            self.document
                .read_in_advancement(&phase, request)
                .map_err(WorthQueryHistoryEditDenial::History)?;
            let current = WorthQueryApplicationRequest {
                application: runtime,
                principal: request.principal,
                scope: request.scope,
                branch: self.document.owner.branch(),
            };
            let Phase::Admission(state) = self.phase.as_mut().expect("admission phase") else {
                unreachable!()
            };
            let (prepared, selected) = <I::Binding as SourceAssessmentDriver<S>>::prepare_history(
                state.operation.as_ref().expect("original action"),
                &phase,
                &current,
            )
            .map_err(|cause| {
                WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Mutation(
                    cause,
                ))
            })?;
            let installed = runtime
                .installed_schema()
                .installed_history::<H>()
                .map_err(|cause| {
                    WorthQueryHistoryEditDenial::History(
                        WorthQueryApplicationHistoryDenial::Installation(cause),
                    )
                })?;
            match self
                .document
                .owner
                .reserve_history_record::<I::Binding>(
                    &phase,
                    &program_publication_access(),
                    &installed,
                    WorthQueryDocumentHistoryOperation::new(
                        &selected,
                        &prepared.admission,
                        prepared.idempotency,
                    ),
                )
                .map_err(|cause| {
                    WorthQueryHistoryEditDenial::History(
                        WorthQueryApplicationHistoryDenial::Document(cause),
                    )
                })? {
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
        self.after_opening(result)
    }

    pub(super) fn advance_fork(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        let Some(Phase::Forking(state)) = self.phase.as_mut() else {
            unreachable!()
        };
        if let Some(recovery) = state.recovery.take() {
            if let Err(next) = recovery.release() {
                state.recovery = Some(next);
                return WorthQueryHistoryEditAdvance::Refused {
                    edit: self,
                    denial: WorthQueryHistoryEditDenial::ForkRecovery,
                };
            }
        }
        let runtime = self.document.owner.application();
        let result = runtime.with_application_advancement(request.scope, |phase| {
            self.document
                .read_in_advancement(&phase, request)
                .map_err(WorthQueryHistoryEditDenial::History)?;
            let current = WorthQueryApplicationRequest {
                application: runtime,
                principal: request.principal,
                scope: request.scope,
                branch: self.document.owner.branch(),
            };
            let Some(Phase::Forking(state)) = self.phase.as_mut() else {
                unreachable!()
            };
            let (prepared, selected) = <I::Binding as SourceAssessmentDriver<S>>::prepare_history(
                state.operation.as_ref().expect("original initial action"),
                &phase,
                &current,
            )
            .map_err(|cause| {
                WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Mutation(
                    cause,
                ))
            })?;
            let installed = runtime
                .installed_schema()
                .installed_history::<H>()
                .map_err(|cause| {
                    WorthQueryHistoryEditDenial::History(
                        WorthQueryApplicationHistoryDenial::Installation(cause),
                    )
                })?;
            self.document
                .owner
                .readmit_history_record::<I::Binding>(
                    &phase,
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
                match self.document.owner.create_history_candidate(
                    &phase,
                    &program_publication_access(),
                    state.reservation.as_ref().unwrap(),
                ) {
                    Ok(candidate) => state.candidate = Some(candidate),
                    Err(WorthQueryDocumentHistoryCandidateDenial::Creation(
                        WorthQueryProductBranchCreateError::ProductUnpublished(recovery),
                    )) => {
                        state.recovery = Some(ForkRecovery::Creation(recovery));
                        return Err(WorthQueryHistoryEditDenial::ForkRecovery);
                    }
                    Err(cause) => return Err(WorthQueryHistoryEditDenial::Candidate(cause)),
                }
            }
            Ok(())
        });
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
            match state
                .operation
                .take()
                .unwrap()
                .bind_history_candidate(restriction)
            {
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
        result: Result<
            Result<(), WorthQueryHistoryEditDenial>,
            worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
        >,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        match result {
            Err(cause) => WorthQueryHistoryEditAdvance::Refused {
                edit: self,
                denial: WorthQueryHistoryEditDenial::History(
                    WorthQueryApplicationHistoryDenial::ExecutionRequest(cause),
                ),
            },
            Ok(Err(denial)) => WorthQueryHistoryEditAdvance::Refused { edit: self, denial },
            Ok(Ok(()))
                if matches!(
                    self.posture(),
                    WorthQueryHistoryEditPosture::AlreadyRecorded
                ) =>
            {
                WorthQueryHistoryEditAdvance::Ready(self)
            }
            Ok(Ok(())) => WorthQueryHistoryEditAdvance::Progress(self),
        }
    }
}
