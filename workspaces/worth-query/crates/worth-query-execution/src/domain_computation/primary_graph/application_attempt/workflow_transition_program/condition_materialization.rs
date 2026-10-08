use super::*;
use crate::domain_computation::primary_graph::application_attempt::check_request_live;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn materialize_condition_requirement(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        retire_live_membership: bool,
        facts: Vec<super::super::WorthQueryApplicationObservedFact>,
        condition: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowCondition,
        allowance: crate::domain_computation::primary_graph::application_attempt::WorkflowStepAllowance,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        self.append_completed_facts(
            facts,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        let subject = self.admission.scope_entity_id();
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            retire_live_membership,
            allowance,
        );
        let required = RequiredWorkflowCondition::from_selected(
            admitted.instance(),
            admitted.node_path().to_owned(),
            admitted.identity().to_owned(),
            admitted.occurrence(),
            condition,
        );
        Ok(PreparedWorkflowAdvance::AwaitingCondition(
            PreparedWorkflowCondition {
                admitted,
                required,
                layout: layout.clone(),
                program_revision,
                replays: Default::default(),
            },
        ))
    }
}
