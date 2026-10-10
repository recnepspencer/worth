//! Projection, admission and candidate construction stay on the advancement owner.
use super::super::WorthQueryDecisionPlan;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;

type Report<Schema, Binding> = WorthQueryMutationHandlerExecutionReport<
    WorthQueryCompletedMutationCandidate<Schema, Binding>,
    <Binding as ApplicationMutationBinding<Schema>>::Denial,
>;
type Completion<'authority, Schema, Binding> = Box<
    dyn FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            &ApplicationMutationIdentities<'_, Schema, Binding>,
            HandlerResult<
                <Binding as ApplicationMutationBinding<Schema>>::Decision,
                <Binding as ApplicationMutationBinding<Schema>>::Denial,
            >,
        ) -> Report<Schema, Binding>
        + 'authority,
>;

pub(in crate::domain_computation::primary_graph) struct PreparedMutationHandler<
    'advance,
    'authority,
    Schema,
    Binding,
> where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    plan: WorthQueryDecisionPlan<Binding::Decision, Binding::Denial>,
    execution: &'advance QueryRequestExecution<'advance>,
    complete: Completion<'authority, Schema, Binding>,
}
impl<'advance, 'authority, Schema, Binding>
    PreparedMutationHandler<'advance, 'authority, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(super) fn new(
        plan: WorthQueryDecisionPlan<Binding::Decision, Binding::Denial>,
        execution: &'advance QueryRequestExecution<'advance>,
        complete: impl FnOnce(
                &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
                &ApplicationMutationIdentities<'_, Schema, Binding>,
                HandlerResult<Binding::Decision, Binding::Denial>,
            ) -> Report<Schema, Binding>
            + 'authority,
    ) -> Self {
        Self {
            plan,
            execution,
            complete: Box::new(complete),
        }
    }

    pub(super) fn run_serial(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    ) -> Report<Schema, Binding> {
        let decision = self.plan.run_serial(self.execution);
        (self.complete)(runtime, identities, decision)
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(self) -> (
        Vec<crate::domain_computation::primary_graph::application_contribution::computation::prepared_request::PreparedCompute>,
        MutationCompletion<'authority, Schema, Binding>,
    ){
        let Self {
            plan,
            execution: _,
            complete,
        } = self;
        let (requests, decision) = plan.into_parts();
        (requests, MutationCompletion { decision, complete })
    }
}

pub(in crate::domain_computation::primary_graph) struct MutationCompletion<
    'authority,
    Schema,
    Binding,
> where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    decision: super::super::decision_plan::DecisionCompletion<Binding::Decision, Binding::Denial>,
    complete: Completion<'authority, Schema, Binding>,
}

impl<Schema, Binding> MutationCompletion<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        slots: Vec<crate::domain_computation::primary_graph::application_contribution::computation::prepared_request::ComputedSlot>,
    ) -> Report<Schema, Binding> {
        (self.complete)(runtime, identities, self.decision.finish(slots))
    }
}
