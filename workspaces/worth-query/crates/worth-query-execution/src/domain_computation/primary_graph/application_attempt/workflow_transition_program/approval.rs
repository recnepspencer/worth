use worth_query_installation::facade::ApplicationSchema;

use super::{PreparedWorkflowAdvance, RequiredWorkflowApproval};
use crate::domain_computation::primary_graph::workflow::instance::{
    SelectedWorkflowApproval, SelectedWorkflowTransition,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::primary_graph::{
    PublishedWorkflowInstanceRef, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};

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
{
    pub(super) fn materialize_approval_requirement(
        mut self,
        layout: &WorthQueryWorkflowLayout,
        instance: PublishedWorkflowInstanceRef,
        selected: SelectedWorkflowTransition,
        facts: Vec<WorthQueryApplicationObservedFact>,
        approval: SelectedWorkflowApproval,
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
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.admission.operation(),
            ));
        }
        self.facts.extend(facts);
        let required =
            RequiredWorkflowApproval::from_selected(instance.entity_id(), &selected, approval);
        Ok(PreparedWorkflowAdvance::AwaitingApproval {
            read_set: self,
            transition_identity_locator: layout.transition.identity.clone(),
            assessment_identity_locator: layout.assessment_evidence.identity.clone(),
            instance: instance.entity_id(),
            required,
            replays: Default::default(),
        })
    }
}
