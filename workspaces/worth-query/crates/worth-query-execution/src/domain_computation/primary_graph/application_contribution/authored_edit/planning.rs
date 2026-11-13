//! One-use typed candidates retained before the independent authored prefix.
use super::capture::TypedStage;
use super::*;
use crate::domain_computation::primary_graph::{WorthQueryCompletedMutationCandidate, WorthQueryMutationHandlerWork};
pub(super) use crate::domain_computation::primary_graph::application_attempt::effect_program::candidate_planning::CandidateFootprint as Footprint;
type Candidate<S, B> = (
    WorthQueryCompletedMutationCandidate<S, B>,
    WorthQueryMutationHandlerWork,
);
type PrefixCandidate<S, B> = (
    HandlerResult<
        WorthQueryCompletedMutationCandidate<S, B>,
        <B as ApplicationMutationBinding<S>>::Denial,
    >,
    WorthQueryMutationHandlerWork,
);
pub(super) struct StagePlanning<S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    pub(super) candidate: Option<Candidate<S, B>>,
    pub(super) support: Option<Footprint>,
}
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> Default for StagePlanning<S, B> {
    fn default() -> Self {
        Self {
            candidate: None,
            support: None,
        }
    }
}
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> TypedStage<S, B>
where
    B::Input: Clone + Send + Sync,
    B::IdempotencyKey: Send + Sync,
    B::Result: Send,
    B::Denial: Send,
{
    pub(super) fn plan_candidate(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        unseen: &UnseenAuthoredEnvelope<'_>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if !context
            .application
            .mutation_handlers
            .supports_candidate_planning::<B>()
        {
            return Ok(());
        }
        let execution = context
            .phase
            .checked_execution_request_for(context.application.product_runtime())
            .map_err(|cause| self.planning_denial(format!("stage execution: {cause:?}")))?;
        let (outcome, work) = self.run_on_c(context, unseen)?;
        match outcome {
            HandlerResult::Completed(candidate) => {
                if let Some(support) =
                    candidate.candidate_footprint(&execution).map_err(|cause| {
                        self.planning_denial(format!("candidate footprint: {cause:?}"))
                    })?
                {
                    let mut planned = self
                        .planning
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    planned.support = Some(support);
                    planned.candidate = Some((candidate, work));
                }
            }
            HandlerResult::DomainDenied(_) => {}
            HandlerResult::Cancelled => {
                return Err(self.planning_denial("candidate planning cancelled"))
            }
            HandlerResult::DeadlineExceeded => {
                return Err(self.planning_denial("candidate planning deadline exceeded"))
            }
            HandlerResult::ExecutionDenied(cause) => {
                return Err(self.planning_denial(format!("candidate planning: {cause:?}")))
            }
        }
        Ok(())
    }
    pub(super) fn run_on_c(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        unseen: &UnseenAuthoredEnvelope<'_>,
    ) -> Result<PrefixCandidate<S, B>, WorthQueryOutputDemandDenial> {
        let admission = (self.admit)(context)?;
        let identities = self
            .key
            .stage_with_encoded_input::<S, B>(&self.input, self.ordinal)
            .map_err(|cause| self.planning_denial(format!("stage identity: {cause:?}")))?;
        let binding = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities)
            .bind_edit_envelope(&unseen.intent)
            .bind_admission(&admission.operation);
        let before = admission
            .operation
            .graph_work()
            .mutation_product()
            .ok_or_else(|| self.planning_denial("missing actual C"))?
            .retained_clone();
        if before.observation() != &self.before
            || admission.operation.scope_entity_id() != self.source
            || !unseen.admits(binding)
        {
            return Err(self.planning_denial("the genuine invocation or C basis changed"));
        }
        let execution = context
            .phase
            .checked_execution_request_for(context.application.product_runtime())
            .map_err(|cause| self.planning_denial(format!("stage execution: {cause:?}")))?;
        let (outcome, work) = context
            .application
            .execute_authored_stage_handler::<B>(
                context.phase,
                &identities,
                &admission.principal,
                &self.principal,
                admission.operation,
                &before,
                None,
                unseen.coordination,
                execution.allocation_policy(),
            )
            .into_parts();
        Ok((
            outcome
                .map_err(|cause| self.planning_denial(format!("candidate planning: {cause:?}")))?,
            work,
        ))
    }

    pub(super) fn planning_denial(
        &self,
        detail: impl Into<String>,
    ) -> WorthQueryOutputDemandDenial {
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::AuthoredStageRefused {
                ordinal: self.ordinal,
                binding: B::IDENTITY,
            },
            detail,
        )
    }
    pub(super) fn take_prefix_candidate(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
        unseen: &UnseenAuthoredEnvelope<'_>,
    ) -> Result<PrefixCandidate<S, B>, WorthQueryOutputDemandDenial> {
        let planned = self
            .planning
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .candidate
            .take();
        let was_planned = planned.is_some();
        let candidate = match planned {
            Some((candidate, work)) => (HandlerResult::Completed(candidate), work),
            None => self.run_on_c(context, unseen)?,
        };
        let execution = context
            .phase
            .checked_execution_request_for(context.application.product_runtime())
            .map_err(|cause| self.planning_denial(format!("stage execution: {cause:?}")))?;
        if was_planned && let HandlerResult::Completed(completed) = &candidate.0 {
            completed
                .require_planned_support(context.application, &execution, None)
                .map_err(|cause| {
                    self.planning_denial(format!("planned prefix support: {cause:?}"))
                })?;
        }
        Ok(candidate)
    }
}
impl<S: ApplicationSchema> CapturedAuthoredEditStage<S> {
    pub fn prepare_prefix_candidate<B: ApplicationMutationBinding<S>>(
        &self,
        _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        unseen: &UnseenAuthoredEnvelope<'_>,
    ) -> Result<PrefixCandidate<S, B>, WorthQueryOutputDemandDenial>
    where
        B::Input: Clone + Send + Sync,
        B::Result: Send,
        B::Denial: Send,
    {
        let candidate = self
            .executor
            .prefix_candidate(&AuthoredEditStageContext { phase, application }, unseen)?;
        candidate
            .downcast::<PrefixCandidate<S, B>>()
            .map(|candidate| *candidate)
            .map_err(|_| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "foreign prefix binding",
                )
            })
    }
}
