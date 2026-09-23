use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryMarkerIdentity,
};
use worth_query_execution::facade::workflow_advance::{
    PreparedWorkflowAdvance, RequiredWorkflowCondition, WorkflowProgressOutcome,
    WorthQueryWorkflowAdvanceAdapter,
};
use worth_query_installation::facade::ApplicationSchema;

use super::progress::{
    WorthQueryWorkflowAdvanceRequest, WorthQueryWorkflowConditionAcceptanceDenial,
};

impl<'application, 'principal, 'scope, Schema, Spec, Program, Operation, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Program,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: worth_query_declaration::facade::application_program::ApplicationWorkflowSpec<
        Schema = Schema,
    >,
    Program:
        worth_query_declaration::facade::application_program::ApplicationProgramDefinition<Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn accept_condition<Binding>(
        self,
        required: &RequiredWorkflowCondition,
        result: crate::domain_computation::WorthQueryPublishedApplicationResult<
            Binding::Query,
            bool,
        >,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowConditionAcceptanceDenial>
    where
        Binding: ApplicationQueryBinding<Schema> + 'static,
        Binding::Query: ApplicationQueryMarkerIdentity<Schema> + 'static,
        <Binding::Query as ApplicationQueryMarkerIdentity<Schema>>::ResultBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = bool,
            >,
    {
        if required.query() != Binding::Query::IDENTIFIER
            || required.parameter_type() != Binding::Query::PARAMETER_TYPE_IDENTITY.as_str()
            || required.result_type() != Binding::Query::RESULT_TYPE_IDENTITY.as_str()
            || required.binding() != Binding::IDENTITY
        {
            return Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch);
        }
        let source = result.into_output_demand_source();
        if let Some(replayed) = WorthQueryWorkflowAdvanceAdapter::resolve_condition_replay(
            self.application,
            &self.prepared,
            required,
            &source,
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowConditionAcceptanceDenial::Replay)?
        {
            return Ok(replayed);
        }
        let prepared = match self.prepared {
            PreparedWorkflowAdvance::AwaitingCondition(prepared) => prepared,
            PreparedWorkflowAdvance::Transition { .. }
            | PreparedWorkflowAdvance::AwaitingAssessment(_)
            | PreparedWorkflowAdvance::AwaitingOperation(_)
            | PreparedWorkflowAdvance::AwaitingEvidence { .. }
            | PreparedWorkflowAdvance::AwaitingApproval { .. }
            | PreparedWorkflowAdvance::ReplayOnly { .. } => {
                return Err(WorthQueryWorkflowConditionAcceptanceDenial::NotAwaitingCondition)
            }
        };
        if !same_requirement(prepared.required(), required) {
            return Err(WorthQueryWorkflowConditionAcceptanceDenial::RequirementMismatch);
        }
        WorthQueryWorkflowAdvanceAdapter::compare_and_commit_condition::<
            Schema,
            Operation,
            Input,
            Scope,
            Binding,
        >(self.application, prepared, source, self.idempotency)
        .map_err(WorthQueryWorkflowConditionAcceptanceDenial::Attempt)
    }
}

fn same_requirement(left: &RequiredWorkflowCondition, right: &RequiredWorkflowCondition) -> bool {
    left.instance() == right.instance()
        && left.node_path() == right.node_path()
        && left.transition_identity() == right.transition_identity()
        && left.occurrence() == right.occurrence()
        && left.query() == right.query()
        && left.parameter_type() == right.parameter_type()
        && left.result_type() == right.result_type()
        && left.binding() == right.binding()
}
