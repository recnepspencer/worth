use super::*;

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
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        if self.facts.len().saturating_add(facts.len())
            > self
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        self.facts.extend(facts);
        let subject = self.admission.scope_entity_id();
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            live_membership,
            retire_live_membership,
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
                replays: Box::default(),
            },
        ))
    }
}
