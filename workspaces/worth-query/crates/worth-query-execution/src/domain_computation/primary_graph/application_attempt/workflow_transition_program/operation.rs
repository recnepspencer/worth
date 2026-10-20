use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIdentities},
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    PreparedWorkflowAdvance, PreparedWorkflowOperation, RequiredWorkflowOperation,
    WorkflowOperationAuthority,
};
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::{
        instance_binding, observe_retained_transition,
    },
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
mod evidence;
use evidence::WorkflowOperationEvidence;
mod receipt;
pub(in crate::domain_computation::primary_graph::application_attempt) use receipt::operation_receipt_requires_recovery;
pub(in crate::domain_computation::primary_graph::application_attempt) use receipt::receipt_identity_from_outcome;
pub(super) use receipt::validate_operation_receipt;
pub(in crate::domain_computation::primary_graph) use receipt::ValidatedWorkflowOperationReceipt;

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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        allowance: crate::domain_computation::primary_graph::application_attempt::WorkflowStepAllowance,
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
        let source_settlement = self.lease.handle().with_runtime(execution, |runtime| {
            observe_retained_transition(
                execution,
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
        check_request_live(self.admission.publication_request(), selected.node_path())?;
        let input_request = self.admission.publication_request();
        let mut input_observation = self.lease.handle().with_runtime(execution, |runtime| {
            observe_workflow_operation_input(
                execution,
                runtime,
                self.lease.snapshot(),
                layout,
                source_transition.entity(),
                source_identity,
                source_operation,
                source_input_type,
                source.path(),
                input_request,
            )
        })?;
        let input_identity = input_observation.input_identity;
        facts.append(&mut input_observation.facts);
        // An external operation records its custody on the instance; the
        // reading it replaces travels with the operation's authority. This
        // advance cannot see the operation's own external posture, so the
        // reading is pinned for every operation; only an external commit
        // ever writes the field, so a local operation is never staled by it.
        let (_, custody) = self.lease.handle().with_runtime(execution, |runtime| {
            instance_binding::owner_custody(
                execution,
                runtime,
                self.lease.snapshot(),
                layout,
                instance.entity_id(),
            )
        })?;
        facts.push(custody);
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        let evidence = self.resolve_operation_evidence(
            execution, compiled, layout, &instance, &selected, progress, &operation, &mut facts,
        )?;
        check_request_live(
            self.admission.publication_request(),
            self.admission.operation(),
        )?;
        let parent_instance = self.lease.handle().with_runtime(execution, |runtime| {
            super::super::workflow_instance_observation::original_parent(
                execution,
                runtime,
                self.lease.snapshot(),
                layout,
                source_transition.entity(),
                &mut facts,
            )
        })?;
        let original_approval = compiled
            .approval_authority_sources(selected.node())
            .next()
            .and_then(|source| progress.latest_transition(source.entity()));
        let approval_parent = original_approval
            .map(|approval| {
                self.lease.handle().with_runtime(execution, |runtime| {
                    super::super::workflow_instance_observation::original_parent(
                        execution,
                        runtime,
                        self.lease.snapshot(),
                        layout,
                        approval.entity(),
                        &mut facts,
                    )
                })
            })
            .transpose()?;
        let original_sources = super::actor_context::OriginalActorSources {
            parent_instance,
            proposal: input_observation.proposal,
            proposal_identity: input_observation.proposal_identity,
            transition: source_transition,
            transition_identity: source_identity.to_owned(),
            approval_parent,
            approval: original_approval,
        };
        authority_facts.extend_from_slice(&facts[handoff_start..]);
        self.append_completed_facts(
            facts,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        let subject = self.admission.scope_entity_id();
        let branch = self.lease.product().product_branch();
        let binding = operation
            .binding
            .clone()
            .ok_or_else(|| mismatch(selected.node_path()))?;
        let runtime_authority = self.admission.runtime_authority().as_u64();
        let session_identity = self.admission.graph_work_session_identity();
        let observation = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            self.lease.product().observation(),
        );
        let handle = self.lease.handle().clone();
        let graph_layout = std::sync::Arc::clone(&self.lease.layout);
        let deadline = self.workflow_deadline;
        let admitted =
            crate::domain_computation::primary_graph::workflow::instance::admit_workflow_transition(
                self,
                selected,
                instance.entity_id(),
                subject,
                live_membership,
                retire_live_membership,
                allowance,
            );
        let authority = match evidence {
            WorkflowOperationEvidence::Satisfied(approval_authority) => Some(
                WorkflowOperationAuthority::new(
                    operation.operation.clone(),
                    binding,
                    *admitted.identity_bytes(),
                    input_identity,
                    runtime_authority,
                    session_identity,
                    observation,
                    authority_facts,
                    admitted.operation_settlement_basis(),
                    layout.clone(),
                    approval_authority,
                    operation.workflow_execution_requirements.clone(),
                    handle,
                    graph_layout,
                )
                .with_deadline(deadline),
            ),
            WorkflowOperationEvidence::Unavailable => None,
        };
        let required = RequiredWorkflowOperation::from_selected(
            branch,
            runtime_authority,
            instance.clone(),
            Some(original_sources),
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
                program_revision: *compiled.program_revision(),
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
        let performed = validate_operation_receipt::<Schema, Binding>(
            runtime,
            &self.required,
            self.admitted.subject(),
            self.admitted.read_set().lease.product().product_branch(),
            receipt,
            None,
        )?;
        let (receipt_identity, executor) = performed.into_parts();
        self.settle_validated(receipt_identity, executor)
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
        let performed = validate_operation_receipt::<Schema, Binding>(
            runtime,
            &self.required,
            self.admitted.subject(),
            self.admitted.read_set().lease.product().product_branch(),
            receipt,
            Some(recovery),
        )?;
        let (receipt_identity, executor) = performed.into_parts();
        self.settle_validated(receipt_identity, executor)
    }

    fn settle_validated(
        self,
        receipt_identity: [u8; 32],
        executor: crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let mut demand = super::PlatformEffectDemand::default();
        visit_workflow_operation_transition_facts(
            &self.layout,
            &self.admitted,
            &receipt_identity,
            executor,
            |effect| demand.observe(&effect),
        )?;
        let reservation = super::admit_platform_effects(self.admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_operation_transition_facts(
            &self.layout,
            &self.admitted,
            &receipt_identity,
            executor,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        reservation.materialize(&effects)?;
        let transition_identity = self.admitted.identity().to_owned();
        let transition_identity_bytes = *self.admitted.identity_bytes();
        let instance = self.admitted.instance();
        let node_path = self.admitted.node_path().to_owned();
        let progress_update = self.admitted.prepare_progress_update(
            worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
            Some(receipt_identity),
        )?;
        let program = WorthQueryApplicationEffectProgram {
            candidate_funding: None,
            candidate_requirements: None,
            effect_backing_memory: None,
            read_set: self.admitted.into_read_set(),
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
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
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

#[cfg(test)]
#[path = "operation/tests.rs"]
mod tests;
