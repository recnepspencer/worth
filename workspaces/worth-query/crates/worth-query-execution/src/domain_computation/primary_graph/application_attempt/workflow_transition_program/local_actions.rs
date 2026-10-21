//! Fresh local actions share exact admission, custody fencing and joint publication.
use super::{
    PreparedWorkflowAdvance, PreparedWorkflowOperation, RequiredWorkflowOperation,
    WorkflowLocalActionKind, WorkflowOperationAuthority,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{PublishedWorkflowInstanceRef, WorkflowStepAllowance},
    workflow::{
        definition::CompiledWorkflowDefinition,
        instance::{
            admit_workflow_transition, SelectedWorkflowOperation, SelectedWorkflowTransition,
        },
        schema::WorthQueryWorkflowLayout,
    },
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::RelationId;

/// A native verified original receipt may settle only its own pending custody.
/// It grants no executable operation authority and has no public constructor.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct VerifiedLocalActionCompletion {
    pub(super) transition: String,
}
#[derive(Clone, Debug)]
pub(super) struct LocalActionDescriptor {
    pub(super) kind: WorkflowLocalActionKind,
    pub(super) spec: String,
    pub(super) support: [u8; 32],
}
impl LocalActionDescriptor {
    pub(super) fn new<S, Spec>(
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec<
            S,
            Spec,
        >,
        kind: WorkflowLocalActionKind,
    ) -> Self
    where
        S: ApplicationSchema,
        Spec: worth_query_declaration::facade::application_program::ApplicationWorkflowSpec<
            Schema = S,
        >,
    {
        Self {
            kind,
            spec: Spec::IDENTITY.as_str().to_owned(),
            support: *installed.support_identity_bytes(),
        }
    }
}

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
    pub(super) fn materialize_observed_local_action(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        layout: &WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        instance: PublishedWorkflowInstanceRef,
        observed: &mut super::super::workflow_instance_observation::ObservedWorkflowInstance,
        requested: &SelectedWorkflowOperation,
        maximum_transitions: usize,
        origin: LocalActionDescriptor,
        membership: RelationId,
        mut facts: Vec<WorthQueryApplicationObservedFact>,
        input_identity: [u8; 32],
        allowance: WorkflowStepAllowance,
        completion: Option<&VerifiedLocalActionCompletion>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let selected = self.select_observed_local_action(
            execution,
            layout,
            compiled,
            &instance,
            observed,
            requested,
            maximum_transitions,
            origin.kind,
            completion,
        )?;
        if completion.is_some_and(|original| selected.identity() != original.transition) {
            return Err(super::denial(
                super::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "local completion's original authored route or frontier changed",
            ));
        }
        facts.append(&mut observed.facts);
        self.materialize_local_action_requirement(
            layout,
            compiled,
            instance,
            selected,
            membership,
            facts,
            requested.clone(),
            input_identity,
            allowance,
            origin,
            completion,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn select_observed_local_action(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        layout: &WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        instance: &PublishedWorkflowInstanceRef,
        observed: &mut super::super::workflow_instance_observation::ObservedWorkflowInstance,
        requested: &SelectedWorkflowOperation,
        maximum_transitions: usize,
        kind: WorkflowLocalActionKind,
        completion: Option<&VerifiedLocalActionCompletion>,
    ) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
        if self.admission.operation() != requested.operation {
            return Err(super::denial(
                super::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                "admitted operation differs from the local workflow action body",
            ));
        }
        self.lease.handle().with_runtime(execution, |runtime| {
            observed.ensure_history(
                execution,
                self.lease.handle(),
                runtime,
                self.lease.snapshot(),
                layout,
                instance.entity_id(),
                maximum_transitions,
                compiled,
            )
        })?;
        let custody = self.lease.handle().with_runtime(execution, |runtime| {
            if let Some(completion) = completion {
                let (custody, fact) = super::super::workflow_instance_observation::instance_binding::owner_custody(
                    execution, runtime, self.lease.snapshot(), layout, instance.entity_id(),
                )?;
                if custody.as_deref() != Some(completion.transition.as_str()) {
                    return Err(super::denial(super::WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
                        "local completion does not own the exact pending occurrence"));
                }
                Ok(fact)
            } else {
            super::super::workflow_instance_observation::instance_binding::fence_owner_custody(
                execution,
                runtime,
                self.lease.snapshot(),
                layout,
                instance.entity_id(),
                &observed.transitions,
                "local workflow action cannot abandon owner custody",
            )
            }
        })?;
        observed.facts.push(custody);
        let select = match kind {
            WorkflowLocalActionKind::Terminate => crate::domain_computation::primary_graph::workflow::instance::select_termination_action_transition,
            WorkflowLocalActionKind::Return => crate::domain_computation::primary_graph::workflow::instance::select_back_action_transition,
        };
        select(
            compiled,
            instance.entity_id(),
            &observed.progress_basis,
            requested,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn materialize_local_action_requirement(
        mut self,
        layout: &WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        instance: PublishedWorkflowInstanceRef,
        selected: SelectedWorkflowTransition,
        membership: RelationId,
        facts: Vec<WorthQueryApplicationObservedFact>,
        operation: SelectedWorkflowOperation,
        input_identity: [u8; 32],
        allowance: WorkflowStepAllowance,
        origin: LocalActionDescriptor,
        completion: Option<&VerifiedLocalActionCompletion>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let authority_facts = facts.clone();
        self.append_completed_facts(
            facts,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )?;
        let subject = self.admission.scope_entity_id();
        let branch = self.lease.product().product_branch();
        let runtime_authority = self.admission.runtime_authority().as_u64();
        let session = self.admission.graph_work_session_identity();
        let observation = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            self.lease.product().observation(),
        );
        let handle = self.lease.handle().clone();
        let graph_layout = std::sync::Arc::clone(&self.lease.layout);
        let deadline = self.workflow_deadline;
        let binding = operation
            .binding
            .clone()
            .expect("validated local workflow action binding");
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            membership,
            matches!(origin.kind, WorkflowLocalActionKind::Terminate),
            allowance,
        );
        let authority = completion.is_none().then(|| {
            WorkflowOperationAuthority::new(
                operation.operation.clone(),
                binding,
                *admitted.identity_bytes(),
                input_identity,
                runtime_authority,
                session,
                observation,
                authority_facts,
                match origin.kind {
                    WorkflowLocalActionKind::Terminate => admitted
                        .operation_settlement_basis()
                        .for_business_termination(),
                    WorkflowLocalActionKind::Return => {
                        admitted.operation_settlement_basis().for_business_return()
                    }
                },
                layout.clone(),
                None,
                operation.workflow_execution_requirements.clone(),
                handle,
                graph_layout,
            )
            .with_deadline(deadline)
        });
        let mut required = RequiredWorkflowOperation::from_selected(
            branch,
            runtime_authority,
            *compiled.program_revision(),
            instance,
            None,
            admitted.node_path().to_owned(),
            admitted.identity().to_owned(),
            *admitted.identity_bytes(),
            admitted.occurrence(),
            operation,
            input_identity,
            authority,
        );
        required.local_action = Some(origin);
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
