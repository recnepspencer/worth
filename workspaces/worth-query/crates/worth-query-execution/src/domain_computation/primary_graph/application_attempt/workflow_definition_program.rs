use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledWorkflowDefinitionParts,
};

use super::effect_program::{admit_platform_effects, PlatformEffectDemand};
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    visit_definition_facts, BoundWorkflowDefinitionContract,
};
mod intent_identity;
mod lineage;
mod publication;

use lineage::select_lineage;
pub use publication::{
    PerformedWorkflowDefinitionPublication, PreparedWorkflowDefinitionPublication,
    PublishedWorkflowDefinitionRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome,
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
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_definition_publication<
        Capability,
        Spec,
        Program,
    >(
        mut self,
        bound: BoundWorkflowDefinitionContract<Schema, Spec, Program>,
        expected_predecessor: WorkflowDefinitionExpectedPredecessor,
    ) -> Result<
        PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if !bound.belongs_to_branch(self.lease.product().product_branch())
            || !bound.belongs_to_occurrence(self.lease.product())
            || !expected_predecessor.belongs_to_branch(self.lease.product().product_branch())
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAffinityMismatch,
                self.admission.operation(),
            ));
        }
        if !bound
            .contract
            .authoring_binding_matches::<Capability, Operation>()
            || self.admission.installed_capability_identity()
                != Some(*bound.contract.authoring_capability_identity_bytes())
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAuthorityMismatch,
                self.admission.operation(),
            ));
        }

        let layout = self.lease.layout.workflow().clone();
        let lineage = self.lease.handle().with_runtime(|runtime| {
            select_lineage::<Spec>(
                runtime,
                self.lease.snapshot(),
                &layout,
                bound.contract.definition(),
                &expected_predecessor,
            )
        })?;
        if self.facts.len().saturating_add(lineage.facts.len())
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
        self.facts.extend(lineage.facts);

        let program_revision = bound.contract.program_revision().clone();
        let WorthQueryInstalledWorkflowDefinitionParts {
            definition,
            assessment_bindings,
            condition_bindings,
            approval_bindings,
        } = bound.contract.into_definition();
        let content_identity = definition.content_identity().clone();
        let workflow_intent_identity =
            intent_identity::workflow_definition_intent_identity::<Spec>(
                definition.identity().as_str(),
                &content_identity,
                &program_revision,
                self.lease.product().product_branch(),
                &expected_predecessor,
                &assessment_bindings,
                &condition_bindings,
                &approval_bindings,
            )
            .map_err(|()| {
                denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionIntentIdentityUnavailable,
                definition.identity().as_str(),
            )
            })?;
        let mut demand = PlatformEffectDemand::default();
        visit_definition_facts(
            &layout,
            &program_revision,
            &definition,
            &assessment_bindings,
            &condition_bindings,
            &approval_bindings,
            lineage.target,
            |effect| demand.observe(&effect),
        )?;
        let reservation = admit_platform_effects(&self, demand)?;
        let mut effects = Vec::new();
        visit_definition_facts(
            &layout,
            &program_revision,
            &definition,
            &assessment_bindings,
            &condition_bindings,
            &approval_bindings,
            lineage.target,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        let validator_work_admission = reservation.materialize(&effects)?;
        let program = WorthQueryApplicationEffectProgram {
            read_set: self,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        };
        Ok(PreparedWorkflowDefinitionPublication {
            program,
            program_revision,
            content_identity,
            content_identity_locator: layout.definition.content_identity.clone(),
            workflow_intent_identity,
        })
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
