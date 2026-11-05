//! Projection, admission and candidate construction stay on the advancement owner.
use super::super::WorthQueryDecisionPlan;
use super::*;
use crate::domain_computation::request_execution::QueryRequestExecution;

type Report<Schema, Binding> = WorthQueryMutationHandlerExecutionReport<
    WorthQueryCompletedMutationCandidate<Schema, Binding>,
    <Binding as ApplicationMutationBinding<Schema>>::Denial,
>;
type Projection<Schema, Binding> = crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, <Binding as ApplicationMutationBinding<Schema>>::Operation>;

type PreparedHandlerParts<'authority, Schema, Binding> = (
    Vec<crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::PreparedCompute>,
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
    projection: Projection<Schema, Binding>,
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

    pub(super) fn run(
        self,
        runtime: &dyn crate::domain_computation::primary_graph::application_entry::host::ApplicationEntryHost<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    ) -> Report<Schema, Binding> {
        let decision = self.plan.run(self.execution);
        (self.complete)(
            runtime.application_runtime(),
            identities,
            decision,
            self.projection,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
        registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
    ) -> Result<
        PreparedHandlerParts<'authority, Schema, Binding>,
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    >{
        let inputs = registry.admit_prepared_inputs(self.projection.prepared_current_inputs())?;
        let Self {
            plan,
            execution: _,
            complete,
            projection,
        } = self;
        let (requests, decision) = plan.into_parts();
        Ok((
            requests,
            MutationCompletion {
                decision,
                complete,
                projection,
            },
            inputs,
        ))
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
    projection: Projection<Schema, Binding>,
}

impl<Schema, Binding> MutationCompletion<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        runtime: &dyn crate::domain_computation::primary_graph::application_entry::host::ApplicationEntryHost<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        slots: Vec<crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::ComputedSlot>,
        execution: &QueryRequestExecution<'_>,
    ) -> Report<Schema, Binding> {
        (self.complete)(
            runtime.application_runtime(),
            identities,
            self.decision.finish(slots, execution),
            self.projection,
        )
    }
}
