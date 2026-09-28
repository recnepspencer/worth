use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_query_installation::facade::ApplicationSchema;

use super::{PreparedWorkflowAdvance, PreparedWorkflowCondition};
use crate::domain_computation::primary_graph::{
    expression::{
        evaluate_condition, supplies, supporting_identity, WorthQueryWorkflowConditionSources,
    },
    workflow::instance::{visit_workflow_transition_facts, WorkflowInstanceState},
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema, Operation, Input, Scope> PreparedWorkflowCondition<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    pub(super) fn settle(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        sources: WorthQueryWorkflowConditionSources<Schema>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let node_path = self.required.node_path().to_owned();
        if !supplies(sources.operands(), self.required.operands()) {
            return Err(denial(&node_path));
        }
        let graph = runtime
            .runtime
            .primary_graph()
            .ok_or_else(|| denial(&node_path))?;
        let selected_product = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            self.admitted.read_set().lease.product().observation(),
        );
        // Every source must be current before any value is read: a stale
        // operand denies as stale, never as an expression result.
        let mut currentness_facts = Vec::new();
        let mut identities = Vec::new();
        let mut declared = Vec::new();
        let mut values = Vec::new();
        for operand in sources.into_operands() {
            let (value, source) = operand.observed.ok_or_else(|| denial(&node_path))?;
            let expected_query_identity = runtime
                .installed_schema()
                .installed_query_identity_by_name(operand.query)
                .ok_or_else(|| denial(&node_path))?;
            identities.push(source.idempotency_identity().bytes());
            currentness_facts.extend(
                source
                    .validate_and_into_facts(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema().binding_identity(),
                        self.admitted.read_set().admission.graph_work_branch(),
                        self.admitted.subject(),
                        &selected_product,
                        operand.query,
                        expected_query_identity,
                        &graph.layout,
                    )
                    .map_err(|_| denial(&node_path))?,
            );
            declared.push((operand.name, operand.expression_type));
            values.push(value);
        }
        super::assessment::ensure_current(
            self.admitted.read_set(),
            &currentness_facts,
            &node_path,
        )?;
        let expression = &self.required.condition.expression;
        let condition_identity = supporting_identity(
            expression,
            declared
                .iter()
                .zip(&identities)
                .map(|((name, _), identity)| (&**name, *identity)),
        );
        let result = declared
            .iter()
            .zip(values)
            .map(|((name, ty), value)| value.map(|value| (&**name, ty, value)))
            .collect::<Result<Vec<_>, _>>()
            .and_then(|operands| evaluate_condition(expression, operands))
            .map_err(|expression| {
                WorthQueryApplicationAttemptDenial::condition_expression(&node_path, expression)
            })?;
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
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
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
