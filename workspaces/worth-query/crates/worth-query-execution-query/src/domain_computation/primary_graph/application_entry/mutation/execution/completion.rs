//! Projection, admission and candidate construction stay on the advancement owner.
use super::super::WorthQueryDecisionPlan;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;

type Report<Schema, Binding> = WorthQueryMutationHandlerExecutionReport<
    WorthQueryCompletedMutationCandidate<Schema, Binding>,
    <Binding as ApplicationMutationBinding<Schema>>::Denial,
>;
type Projection<Schema, Binding> = super::projection::PreparedMutationProjection<
    Schema,
    <Binding as ApplicationMutationBinding<Schema>>::Operation,
>;

type PreparedHandlerParts<'authority, Schema, Binding> = (
    Vec<crate::domain_computation::primary_graph::application_contribution::computation::prepared_request::PreparedCompute>,
    MutationCompletion<'authority, Schema, Binding>,
    crate::domain_computation::primary_graph::application_output_demand::PreparedRequiredInputs,
);

type Completion<'authority, Schema, Binding> = Box<
    dyn FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            &ApplicationMutationIdentities<'_, Schema, Binding>,
            HandlerResult<
                <Binding as ApplicationMutationBinding<Schema>>::Decision,
                <Binding as ApplicationMutationBinding<Schema>>::Denial,
            >,
            Projection<Schema, Binding>,
        ) -> Report<Schema, Binding>
        + 'authority,
>;

pub struct PreparedMutationHandler<'advance, 'authority, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    plan: WorthQueryDecisionPlan<Binding::Decision, Binding::Denial>,
    execution: &'advance QueryRequestExecution<'advance>,
    complete: Completion<'authority, Schema, Binding>,
    projection: Projection<Schema, Binding>,
}
impl<'advance, 'authority, Schema, Binding>
    PreparedMutationHandler<'advance, 'authority, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub fn new(
        plan: WorthQueryDecisionPlan<Binding::Decision, Binding::Denial>,
        execution: &'advance QueryRequestExecution<'advance>,
        projection: Projection<Schema, Binding>,
        complete: impl FnOnce(
                &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
                &ApplicationMutationIdentities<'_, Schema, Binding>,
                HandlerResult<Binding::Decision, Binding::Denial>,
                Projection<Schema, Binding>,
            ) -> Report<Schema, Binding>
            + 'authority,
    ) -> Self {
        Self {
            plan,
            execution,
            complete: Box::new(complete),
            projection,
        }
    }

    pub fn run(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    ) -> Report<Schema, Binding> {
        let decision = self.plan.run(self.execution);
        let proofs = matches!(decision, HandlerResult::DomainDenied(_))
            .then(|| self.projection.refusal_proofs())
            .flatten();
        let report = (self.complete)(runtime, identities, decision, self.projection);
        match proofs {
            Some(proofs) => report.with_refusal_proofs(proofs),
            None => report,
        }
    }

    pub fn into_parts(
        self,
        registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
    ) -> Result<
        PreparedHandlerParts<'authority, Schema, Binding>,
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    > {
        let inputs = registry.admit_prepared_inputs(self.projection.prepared_current_inputs())?;
        let (requests, completion) = self.into_candidate_parts();
        Ok((requests, completion, inputs))
    }

    /// Candidate inputs were admitted against its applied pin, not the committed registry.
    pub fn into_candidate_parts(
        self,
    ) -> (
        Vec<crate::domain_computation::primary_graph::application_contribution::computation::prepared_request::PreparedCompute>,
        MutationCompletion<'authority, Schema, Binding>,
    ){
        let Self {
            plan,
            execution: _,
            complete,
            projection,
        } = self;
        let (requests, decision) = plan.into_parts();
        (
            requests,
            MutationCompletion {
                decision,
                complete,
                projection,
            },
        )
    }
}

pub struct MutationCompletion<'authority, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    decision: super::super::decision_plan::DecisionCompletion<Binding::Decision, Binding::Denial>,
    complete: Completion<'authority, Schema, Binding>,
    projection: Projection<Schema, Binding>,
}

impl<Schema, Binding> MutationCompletion<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub fn finish(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        slots: Vec<crate::domain_computation::primary_graph::application_contribution::computation::prepared_request::ComputedSlot>,
        execution: &QueryRequestExecution<'_>,
    ) -> Report<Schema, Binding> {
        let decision = self.decision.finish(slots, execution);
        let proofs = matches!(decision, HandlerResult::DomainDenied(_))
            .then(|| self.projection.refusal_proofs())
            .flatten();
        let report = (self.complete)(runtime, identities, decision, self.projection);
        match proofs {
            Some(proofs) => report.with_refusal_proofs(proofs),
            None => report,
        }
    }
}
