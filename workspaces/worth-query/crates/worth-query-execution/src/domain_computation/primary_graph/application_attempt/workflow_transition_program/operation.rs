use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    PreparedWorkflowAdvance, PreparedWorkflowOperation, RequiredWorkflowOperation,
    WorkflowOperationAuthority,
};
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::observe_retained_transition,
    workflow::instance::{visit_workflow_operation_transition_facts, WorkflowInstanceProgress},
    workflow::{
        definition::{CompiledWorkflowDefinition, CompiledWorkflowNodeKind},
        proposal::observe_workflow_operation_input,
    },
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProjectedApplicationMutation,
};

mod authority_binding;
mod receipt;
pub(super) use receipt::{operation_receipt_requires_recovery, validate_operation_receipt};

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
    pub(super) fn materialize_operation_requirement(
        mut self,
        layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        instance: super::super::PublishedWorkflowInstanceRef,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransition,
        live_membership: worth_relational::facade::identity::RelationId,
        retire_live_membership: bool,
        mut facts: Vec<WorthQueryApplicationObservedFact>,
        mut authority_facts: Vec<WorthQueryApplicationObservedFact>,
        progress: &WorkflowInstanceProgress,
        operation: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowOperation,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let handoff_start = facts.len();
        let mut sources = compiled.operation_input_sources(selected.node());
        let source = sources
            .next()
            .ok_or_else(|| mismatch(selected.node_path()))?;
        if sources.next().is_some() {
            return Err(mismatch(selected.node_path()));
        }
        let CompiledWorkflowNodeKind::Operation {
            operation: source_operation,
            input_type: source_input_type,
            ..
        } = source.kind()
        else {
            return Err(mismatch(selected.node_path()));
        };
        if source_input_type != &operation.input_type {
            return Err(mismatch(selected.node_path()));
        }
        let source_transition = progress
            .latest_transition(source.entity())
            .ok_or_else(|| mismatch(selected.node_path()))?;
        let source_settlement = self.lease.handle().with_runtime(|runtime| {
            observe_retained_transition(
                runtime,
                self.lease.snapshot(),
                layout,
                source_transition,
                &mut facts,
            )
        })?;
        let source_identity = progress
            .latest_transition_identity(source.entity())
            .ok_or_else(|| mismatch(selected.node_path()))?;
        if source_settlement.occurrence() != source_transition.settlement().occurrence() {
            return Err(mismatch(selected.node_path()));
        }
        let (input_identity, mut input_facts) = self.lease.handle().with_runtime(|runtime| {
            observe_workflow_operation_input(
                runtime,
                self.lease.snapshot(),
                layout,
                source_transition.entity(),
                source_identity,
                source_operation,
                source_input_type,
                source.path(),
            )
        })?;
        facts.append(&mut input_facts);
        let mut approval_sources = compiled.approval_authority_sources(selected.node());
        let approval = approval_sources
            .next()
            .ok_or_else(|| mismatch(selected.node_path()))?;
        if approval_sources.next().is_some()
            || !matches!(approval.kind(), CompiledWorkflowNodeKind::Approval { .. })
        {
            return Err(mismatch(selected.node_path()));
        }
        let approval_transition = progress
            .latest_transition(approval.entity())
            .ok_or_else(|| mismatch(selected.node_path()))?;
        let settlement = self.lease.handle().with_runtime(|runtime| {
            observe_retained_transition(
                runtime,
                self.lease.snapshot(),
                layout,
                approval_transition,
                &mut facts,
            )
        })?;
        if settlement.outcome()
            != worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Approved
        {
            return Err(mismatch(selected.node_path()));
        }
        let remaining = self
            .admission
            .allowed_graph_contract()
            .decision_fact_budget()
            .saturating_sub(self.facts.len().saturating_add(facts.len()));
        let approval_authority = match super::approval_decision::observe_operation_approval_inputs(
            compiled,
            layout,
            &self.lease.layout,
            self.admission.scope_entity_id(),
            &instance,
            progress,
            approval.entity(),
            &operation.operation,
            self.lease.handle(),
            self.lease.snapshot(),
            remaining,
        ) {
            Ok((mut observed, authority)) => {
                facts.append(&mut observed);
                Some(authority)
            }
            Err(denial)
                if matches!(
                    denial.kind(),
                    WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
                        | WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceIncomplete
                ) =>
            {
                None
            }
            Err(denial) => return Err(denial),
        };
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
        authority_facts.extend_from_slice(&facts[handoff_start..]);
        self.facts.extend(facts);
        let subject = self.admission.scope_entity_id();
        let branch = self.lease.product().product_branch();
        let binding = operation
            .binding
            .clone()
            .ok_or_else(|| mismatch(selected.node_path()))?;
        let authority = approval_authority.map(|approval_authority| {
            WorkflowOperationAuthority::new(
                operation.operation.clone(),
                binding,
                *selected.identity_bytes(),
                input_identity,
                self.admission.runtime_authority().as_u64(),
                self.admission.graph_work_session_identity(),
                crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                    self.lease.product().observation(),
                ),
                authority_facts,
                approval_authority,
                self.lease.handle().clone(),
                std::sync::Arc::clone(&self.lease.layout),
            )
        });
        let admitted =
            crate::domain_computation::primary_graph::workflow::instance::admit_workflow_transition(
                self,
                selected,
                instance.entity_id(),
                subject,
                live_membership,
                retire_live_membership,
            );
        let required = RequiredWorkflowOperation::from_selected(
            branch,
            admitted.instance(),
            admitted.node_path().to_owned(),
            admitted.identity().to_owned(),
            *admitted.identity_bytes(),
            admitted.occurrence(),
            operation,
            input_identity,
            authority,
        );
        Ok(PreparedWorkflowAdvance::AwaitingOperation(
            PreparedWorkflowOperation {
                admitted,
                required,
                layout: layout.clone(),
                program_revision: compiled.program_revision().clone(),
                replays: Default::default(),
            },
        ))
    }
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowOperation<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    pub(super) fn settle<Binding>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let receipt_identity = validate_operation_receipt::<Schema, Binding>(
            runtime,
            &self.required,
            self.admitted.subject(),
            self.admitted.read_set().lease.product().product_branch(),
            receipt,
            None,
        )?;
        self.settle_validated(receipt_identity)
    }

    pub(super) fn settle_recovered<Binding>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: &WorthQueryApplicationCommitReceipt,
        recovery: &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let receipt_identity = validate_operation_receipt::<Schema, Binding>(
            runtime,
            &self.required,
            self.admitted.subject(),
            self.admitted.read_set().lease.product().product_branch(),
            receipt,
            Some(recovery),
        )?;
        self.settle_validated(receipt_identity)
    }

    fn settle_validated(
        self,
        receipt_identity: [u8; 32],
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let mut demand = super::PlatformEffectDemand::default();
        visit_workflow_operation_transition_facts(
            &self.layout,
            &self.admitted,
            &receipt_identity,
            |effect| demand.observe(&effect),
        )?;
        let reservation = super::admit_platform_effects(self.admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_operation_transition_facts(
            &self.layout,
            &self.admitted,
            &receipt_identity,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        let validator_work_admission = reservation.materialize(&effects)?;
        let transition_identity = self.admitted.identity().to_owned();
        let transition_identity_bytes = *self.admitted.identity_bytes();
        let instance = self.admitted.instance();
        let node_path = self.admitted.node_path().to_owned();
        let progress_update = self.admitted.prepare_progress_update(
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
            Some(receipt_identity),
        )?;
        let program = WorthQueryApplicationEffectProgram {
            read_set: self.admitted.into_read_set(),
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
            output_currentness_facts: None,
        };
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision: self.program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: self.layout.transition.identity.clone(),
            assessment_identity_locator: self.layout.assessment_evidence.identity.clone(),
            instance,
            node_path,
            assessment: None,
            supporting_identity: Some(receipt_identity),
            operation_receipt_identity: Some(receipt_identity),
            progress_update: Some(progress_update),
            terminal: false,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: self.replays,
        })
    }
}

fn mismatch(subject: impl Into<String>) -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}

#[cfg(test)]
#[path = "operation/tests.rs"]
mod tests;
