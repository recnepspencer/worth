use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowControlOutcome,
    application_query::{ApplicationQueryBinding, ApplicationQueryMarkerIdentity},
};
use worth_query_installation::facade::ApplicationSchema;

use super::{PreparedWorkflowAdvance, PreparedWorkflowCondition};
use crate::domain_computation::primary_graph::{
    workflow::instance::{visit_workflow_transition_facts, WorkflowInstanceState},
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationOutputDemandSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema, Operation, Input, Scope> PreparedWorkflowCondition<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    pub(super) fn settle<Binding>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        source: WorthQueryApplicationOutputDemandSource<Binding::Query, bool>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Binding: ApplicationQueryBinding<Schema> + 'static,
        Binding::Query: ApplicationQueryMarkerIdentity<Schema> + 'static,
    {
        if self.required.query() != Binding::Query::IDENTIFIER
            || self.required.parameter_type() != Binding::Query::PARAMETER_TYPE_IDENTITY.as_str()
            || self.required.result_type() != Binding::Query::RESULT_TYPE_IDENTITY.as_str()
            || self.required.binding() != Binding::IDENTITY
        {
            return Err(denial(self.required.node_path()));
        }
        let (result, source) = source
            .into_single_source()
            .ok_or_else(|| denial(self.required.node_path()))?;
        let graph = runtime
            .runtime
            .primary_graph()
            .ok_or_else(|| denial(self.required.node_path()))?;
        let expected_query_identity = runtime
            .installed_schema()
            .installed_query_identity_by_name(self.required.query())
            .ok_or_else(|| denial(self.required.node_path()))?;
        let selected_product = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            self.admitted.read_set().lease.product().observation(),
        );
        let condition_identity = source.idempotency_identity().bytes();
        let currentness_facts = source
            .validate_and_into_facts(
                runtime.runtime.authority_identity().as_u64(),
                &runtime.installed_schema().binding_identity(),
                self.admitted.read_set().admission.graph_work_branch(),
                self.admitted.subject(),
                &selected_product,
                self.required.query(),
                expected_query_identity,
                &graph.layout,
            )
            .map_err(|_| denial(self.required.node_path()))?;
        let currentness_facts: std::sync::Arc<[_]> = currentness_facts.into();
        let outcome = if result {
            ApplicationWorkflowControlOutcome::ConditionSatisfied
        } else {
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied
        };
        let mut demand = super::PlatformEffectDemand::default();
        visit_workflow_transition_facts(
            &self.layout,
            &self.admitted,
            outcome,
            WorkflowInstanceState::Ready,
            |effect| demand.observe(&effect),
        )?;
        let reservation = super::admit_platform_effects(self.admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_transition_facts(
            &self.layout,
            &self.admitted,
            outcome,
            WorkflowInstanceState::Ready,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        let validator_work_admission = reservation.materialize(&effects)?;
        let transition_identity = self.admitted.identity().to_owned();
        let transition_identity_bytes = *self.admitted.identity_bytes();
        let node_path = self.admitted.node_path().to_owned();
        let progress_update = self.admitted.prepare_progress_update(outcome, None)?;
        let mut read_set = self.admitted.into_read_set();
        super::assessment::bind_currentness_facts(&mut read_set, &currentness_facts, &node_path)?;
        let program = WorthQueryApplicationEffectProgram {
            read_set,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            platform_mutation: true,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: Some(currentness_facts),
        };
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision: self.program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: self.layout.transition.identity.clone(),
            assessment_identity_locator: self.layout.assessment_evidence.identity.clone(),
            instance: self.required.instance(),
            node_path,
            assessment: None,
            supporting_identity: Some(condition_identity),
            operation_receipt_identity: None,
            progress_update: Some(progress_update),
            terminal: false,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: self.replays,
        })
    }
}

fn denial(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
