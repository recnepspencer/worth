use worth_query_declaration::facade::{
    application_program::{ApplicationExpressionOperandValue, ApplicationWorkflowSpec},
    application_query::{ApplicationQueryBinding, ApplicationQueryMarkerIdentity},
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_execution::publication_boundary::workflow_advance::{
    PreparedWorkflowAdvance, RequiredWorkflowCondition, WorkflowProgressOutcome,
    WorthQueryWorkflowAdvanceAdapter, WorthQueryWorkflowConditionSources,
};
use worth_query_installation::facade::ApplicationSchema;

use super::progress::{
    WorthQueryWorkflowAdvanceRequest, WorthQueryWorkflowConditionAcceptanceDenial,
};

/// A condition being accepted: each operand's published result is supplied
/// by name, then [`accept`](Self::accept) evaluates the condition's
/// expression over them and settles the transition it selects.
pub struct WorthQueryWorkflowConditionAcceptance<
    'application,
    'principal,
    'scope,
    'required,
    Schema,
    Spec,
    Operation,
    Input,
    Scope,
> where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    request: WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >,
    required: &'required RequiredWorkflowCondition,
    sources: WorthQueryWorkflowConditionSources<Schema>,
}

impl<'application, 'principal, 'scope, Schema, Spec, Operation, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    /// Begins accepting `required`, the condition this request awaits.
    pub fn condition<'required>(
        self,
        required: &'required RequiredWorkflowCondition,
    ) -> WorthQueryWorkflowConditionAcceptance<
        'application,
        'principal,
        'scope,
        'required,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    > {
        WorthQueryWorkflowConditionAcceptance {
            request: self,
            required,
            sources: WorthQueryWorkflowConditionSources::new(),
        }
    }
}

impl<'application, 'principal, 'scope, 'required, Schema, Spec, Operation, Input, Scope>
    WorthQueryWorkflowConditionAcceptance<
        'application,
        'principal,
        'scope,
        'required,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    /// Supplies operand `name` from `Binding`'s published result, which must
    /// be exactly one row.
    pub fn operand<Binding, Value>(
        mut self,
        name: &str,
        result: crate::domain_computation::WorthQueryPublishedApplicationResult<
            Binding::Query,
            Value,
        >,
    ) -> Self
    where
        Binding: ApplicationQueryBinding<Schema>,
        Binding::Query: ApplicationQueryMarkerIdentity<Schema>,
        <Binding::Query as ApplicationQueryMarkerIdentity<Schema>>::ResultBinding:
            ApplicationStructuredValueBinding<Value = Value>,
        Value: ApplicationExpressionOperandValue,
    {
        self.sources = self
            .sources
            .operand::<Binding, Value>(name, result.into_output_demand_source());
        self
    }

    /// Settles the condition. The supplied operands must be exactly the
    /// required ones; true and false select their successors, and a denied
    /// expression selects neither.
    pub fn accept(
        self,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowConditionAcceptanceDenial> {
        let Self {
            request,
            required,
            sources,
        } = self;
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                if !sources.supplies(required) {
                    return Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch);
                }
                if let Some(replayed) = WorthQueryWorkflowAdvanceAdapter::resolve_condition_replay(
                    request.application,
                    &request.prepared,
                    required,
                    &sources,
                    request.idempotency,
                )
                .map_err(WorthQueryWorkflowConditionAcceptanceDenial::Replay)?
                {
                    return Ok(replayed);
                }
                let prepared = match request.prepared {
                    PreparedWorkflowAdvance::AwaitingCondition(prepared) => prepared,
                    PreparedWorkflowAdvance::Transition { .. }
                    | PreparedWorkflowAdvance::AwaitingAssessment(_)
                    | PreparedWorkflowAdvance::AwaitingOperation(_)
                    | PreparedWorkflowAdvance::AwaitingEvidence { .. }
                    | PreparedWorkflowAdvance::AwaitingApproval { .. }
                    | PreparedWorkflowAdvance::ReplayOnly { .. } => {
                        return Err(
                            WorthQueryWorkflowConditionAcceptanceDenial::NotAwaitingCondition,
                        )
                    }
                };
                if prepared.required() != required {
                    return Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch);
                }
                WorthQueryWorkflowAdvanceAdapter::compare_and_commit_condition(
                    &phase,
                    request.application,
                    prepared,
                    sources,
                    request.idempotency,
                )
                .map_err(WorthQueryWorkflowConditionAcceptanceDenial::Attempt)
            })
            .unwrap_or_else(|cause| {
                Ok(WorkflowProgressOutcome::Application(
                    cause
                        .into_commit_outcome()
                        .landed()
                        .expect_err("request admission cannot commit"),
                ))
            })
    }
}
