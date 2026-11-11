//! Each retry creates real C admission and reads D through the shared handler path.
use super::capture::{StageProgram, TypedStage};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    self, CommitPreparedCoupledWriter, InspectedAuthoredApplication, InspectedCoupledWriter,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
impl<S, B> AuthoredEditStageExecutor<S> for TypedStage<S, B>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + Send + Sync,
    B::IdempotencyKey: Send + Sync,
    B::Result: Send,
    B::Denial: Send,
{
    fn prepare<'run>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &AuthoredEditStageContext<'_, 'run, '_, S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
        before: &crate::basis::WorthQueryProductBranchLease,
        before_pin: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<InspectedAuthoredApplication<'run>, WorthQueryOutputDemandDenial> {
        // An abandoned cold candidate's successful value is provisional. Keep
        // a real refusal sticky, but never let an old success hide a later stop.
        {
            let mut retained = self
                .result
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if retained
                .as_ref()
                .is_some_and(|value| value.is::<RetainedAuthoredStageResult<B::Result>>())
            {
                *retained = None;
            }
        }
        let refuse = |detail: String| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::AuthoredStageRefused {
                    ordinal: self.ordinal,
                    binding: B::IDENTITY,
                },
                detail,
            )
        };
        let stopped = |cause: WorthQueryApplicationCommitOutcome| {
            let denial = refuse(format!("stage protocol: {cause:?}"));
            match cause.landed() {
                Err(cause) => denial.with_uncommitted(cause),
                Ok(_) => denial,
            }
        };
        let admission = (self.admit)(context)?;
        if admission.operation.scope_entity_id() != self.source {
            return Err(refuse(
                "the admitted stage source changed before candidate execution".into(),
            ));
        }
        let identities = self
            .key
            .stage_with_encoded_input::<S, B>(&self.input, self.ordinal)
            .map_err(|cause| refuse(format!("stage identity: {cause:?}")))?;
        let report = context.application.execute_authored_stage_handler::<B>(
            context.phase,
            &identities,
            &admission.principal,
            &self.principal,
            admission.operation,
            before,
            before_pin,
            candidate,
            coordination,
            execution.allocation_policy(),
        );
        let stopped_stage = |cause| {
            self.result
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_or_insert_with(|| {
                    Box::new(AuthoredStageExecutionDenial {
                        identity: *identities.edit_stage_identity().expect("canonical stage"),
                        binding: B::IDENTITY,
                        cause,
                    })
                });
            refuse("stage execution refused".into())
        };
        let completed = match report
            .into_outcome()
            .map_err(|cause| stopped_stage(AuthoredStageExecutionCause::Request(cause)))?
        {
            HandlerResult::Completed(completed) => completed,
            HandlerResult::DomainDenied(denial) => {
                self.result
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get_or_insert_with(|| {
                        Box::new(AuthoredStageDomainDenial {
                            identity: *identities.edit_stage_identity().expect("canonical stage"),
                            binding: B::IDENTITY,
                            denial: Box::new(denial),
                        })
                    });
                return Err(refuse("domain denial".into()));
            }
            HandlerResult::Cancelled => {
                return Err(stopped_stage(AuthoredStageExecutionCause::Cancelled))
            }
            HandlerResult::DeadlineExceeded => {
                return Err(stopped_stage(AuthoredStageExecutionCause::DeadlineExceeded))
            }
            HandlerResult::ExecutionDenied(cause) => {
                return Err(stopped_stage(AuthoredStageExecutionCause::Handler(cause)))
            }
        };
        let (program, result) = completed.into_parts();
        let program: StageProgram<S, B> = (self.seed)(execution, program);
        let binding =
            WorthQueryApplicationIdempotencyBinding::for_authored_stage_identities(&identities)
                .expect("canonical stage identities");
        let prepared = application_attempt::prepare_coupled_application::<S, B>(
            execution,
            context.phase,
            context.application,
            coordination,
            program,
            binding,
            None,
        )
        .map_err(stopped)?;
        let registered = application_attempt::register_coupled_application::<S, B>(
            execution,
            context.phase,
            context.application,
            prepared,
        )
        .map_err(stopped)?;
        let inspected = application_attempt::inspect_authored_application::<S, B>(
            execution,
            context.phase,
            context.application,
            registered,
            candidate,
        )
        .map_err(stopped)?;
        *self
            .result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(Box::new(RetainedAuthoredStageResult {
                result,
                identity: *identities.edit_stage_identity().expect("canonical stage"),
                receipt: None,
            }));
        Ok(inspected)
    }
    fn prepare_commit<'run>(
        &self,
        writer: InspectedCoupledWriter<'run>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
    ) -> Result<CommitPreparedCoupledWriter<'run>, WorthQueryApplicationCommitOutcome> {
        writer
            .approve(execution, context.phase, context.application)?
            .prepare::<S, B>(execution, context.phase, context.application)
    }
    fn authorize(
        &self,
        writer: &CommitPreparedCoupledWriter<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        writer.authorize::<S, B>(execution, application, coordination)
    }
    fn complete(
        &self,
        writer: CommitPreparedCoupledWriter<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        unpublished: Option<&crate::domain_computation::WorthQueryProductUnpublishedApplication>,
    ) -> WorthQueryApplicationCommitOutcome {
        let outcome =
            writer.complete::<S, B>(execution, context.phase, context.application, unpublished);
        if let WorthQueryApplicationCommitOutcome::Committed(receipt)
        | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) = &outcome
            && let Some(value) = self
                .result
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_mut()
                .and_then(|value| value.downcast_mut::<RetainedAuthoredStageResult<B::Result>>())
        {
            value.receipt = Some(receipt.clone());
        }
        outcome
    }
}
