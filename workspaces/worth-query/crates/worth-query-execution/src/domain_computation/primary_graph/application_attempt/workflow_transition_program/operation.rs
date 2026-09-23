use sha2::{Digest, Sha256};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use super::{PreparedWorkflowAdvance, PreparedWorkflowOperation, RequiredWorkflowOperation};
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::{
        latest_transition_for_node, ObservedWorkflowTransition,
    },
    workflow::instance::visit_workflow_operation_transition_facts,
    workflow::{
        definition::{CompiledWorkflowDefinition, CompiledWorkflowNodeKind},
        instance::select_settled_replay_transition,
        proposal::observe_workflow_operation_input,
    },
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProjectedApplicationMutation,
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
        transitions: &[ObservedWorkflowTransition],
        operation: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowOperation,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
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
        let source_transition = latest_transition_for_node(transitions, source.entity())
            .ok_or_else(|| mismatch(selected.node_path()))?;
        let source_replay = select_settled_replay_transition(
            compiled,
            instance.entity_id(),
            source_transition.settlement,
        )?;
        let (input_identity, mut input_facts) = self.lease.handle().with_runtime(|runtime| {
            observe_workflow_operation_input(
                runtime,
                self.lease.snapshot(),
                layout,
                source_transition.entity,
                source_replay.identity(),
                source_operation,
                source_input_type,
                source.path(),
            )
        })?;
        facts.append(&mut input_facts);
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
            admitted.instance(),
            admitted.node_path().to_owned(),
            admitted.identity().to_owned(),
            *admitted.identity_bytes(),
            admitted.occurrence(),
            operation,
            input_identity,
        );
        Ok(PreparedWorkflowAdvance::AwaitingOperation(
            PreparedWorkflowOperation {
                admitted,
                required,
                layout: layout.clone(),
                program_revision: compiled.program_revision().clone(),
                replays: Box::default(),
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
        )?;
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
            approval: None,
            approval_identity: None,
            replays: self.replays,
        })
    }
}

pub(super) fn validate_operation_receipt<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    required: &RequiredWorkflowOperation,
    subject: worth_relational::facade::identity::EntityId,
    branch: crate::basis::WorthQueryProductBranch,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Result<[u8; 32], WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    if required.operation() != Binding::Operation::IDENTIFIER
        || required.input_type() != Binding::InputBinding::IDENTITY.as_str()
    {
        return Err(mismatch(required.node_path()));
    }
    let installed = runtime
        .installed_schema()
        .installed_operation(ApplicationOperationRef::<
            Schema,
            Binding::Operation,
            Binding::Input,
        >::from_declaration())
        .map_err(|_| mismatch(required.node_path()))?;
    let scope = receipt.authority_binding().principal_scope().scope();
    let receipt_idempotency = receipt.authority_binding().idempotency_binding();
    if receipt.runtime_authority() != runtime.runtime.authority_identity()
        || receipt
            .authority_binding()
            .principal_scope()
            .binding_identity()
            != &runtime.installed_schema().binding_identity()
        || receipt.installed_operation() != &installed.authority_identity_bytes()
        || receipt.product_branch() != branch
        || scope.partition_id() != subject.partition_value()
        || scope.local_slot() != subject.local_slot_value()
        || scope.generation() != subject.generation_value()
        || !receipt_idempotency.matches_workflow_operation(required.transition_identity_bytes())
        || receipt_idempotency.intent_identity() != required.input_identity()
    {
        return Err(mismatch(required.node_path()));
    }
    receipt_identity(receipt).ok_or_else(|| mismatch(required.node_path()))
}

fn receipt_identity(receipt: &WorthQueryApplicationCommitReceipt) -> Option<[u8; 32]> {
    let mut digest = Sha256::new();
    digest.update(b"worth-query.workflow-operation-receipt.v1");
    digest.update(receipt.runtime_authority().as_u64().to_le_bytes());
    digest.update(receipt.outcome_identity()?.get().to_le_bytes());
    digest.update(receipt.installed_operation());
    Some(digest.finalize().into())
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
