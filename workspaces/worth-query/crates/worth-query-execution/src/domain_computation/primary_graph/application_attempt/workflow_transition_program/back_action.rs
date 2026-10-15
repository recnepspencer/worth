//! Fresh Back authority reuses guarded body execution and local atomic settlement.
use super::{
    PreparedWorkflowAdvance, PreparedWorkflowOperation, RequiredWorkflowOperation,
    WorkflowOperationAuthority,
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
    pub(super) fn select_observed_back_action(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        layout: &WorthQueryWorkflowLayout,
        compiled: &CompiledWorkflowDefinition,
        instance: &PublishedWorkflowInstanceRef,
        observed: &mut super::super::workflow_instance_observation::ObservedWorkflowInstance,
        requested: &SelectedWorkflowOperation,
        maximum_transitions: usize,
    ) -> Result<SelectedWorkflowTransition, WorthQueryApplicationAttemptDenial> {
        if self.admission.operation() != requested.operation {
            return Err(super::denial(
                super::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                "admitted operation differs from Back body",
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
            super::super::workflow_instance_observation::instance_binding::fence_owner_custody(
                execution,
                runtime,
                self.lease.snapshot(),
                layout,
                instance.entity_id(),
                &observed.transitions,
                "business Back cannot abandon owner custody",
            )
        })?;
        observed.facts.push(custody);
        crate::domain_computation::primary_graph::workflow::instance::select_back_action_transition(
            compiled,
            instance.entity_id(),
            &observed.progress_basis,
            requested,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn materialize_back_action_requirement(
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
            .expect("validated Back route binding");
        let admitted = admit_workflow_transition(
            self,
            selected,
            instance.entity_id(),
            subject,
            membership,
            false,
            allowance,
        );
        let authority = WorkflowOperationAuthority::new(
            operation.operation.clone(),
            binding,
            *admitted.identity_bytes(),
            input_identity,
            runtime_authority,
            session,
            observation,
            authority_facts,
            admitted.operation_settlement_basis().for_business_return(),
            layout.clone(),
            None,
            operation.workflow_execution_requirements.clone(),
            handle,
            graph_layout,
        )
        .with_deadline(deadline);
        let required = RequiredWorkflowOperation::from_selected(
            branch,
            admitted.instance(),
            admitted.node_path().to_owned(),
            admitted.identity().to_owned(),
            *admitted.identity_bytes(),
            admitted.occurrence(),
            operation,
            input_identity,
            Some(authority),
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
