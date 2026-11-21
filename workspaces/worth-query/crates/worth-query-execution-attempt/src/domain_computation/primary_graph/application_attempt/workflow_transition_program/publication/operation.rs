#[path = "operation/actor_origin.rs"]
pub mod actor_origin;
#[path = "operation/approval_denial.rs"]
pub mod approval_denial;
use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use std::sync::{Arc, Mutex};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::{
    application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease,
    WorthQueryPrimaryGraphIntegrationHandle,
};

use super::PreparedWorkflowTransitionReplays;

pub struct PreparedWorkflowOperation<Schema, Operation, Input, Scope> {
    pub admitted:
        crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    pub required:
        RequiredWorkflowOperation,
    pub layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub program_revision:
        ApplicationProgramRevision,
    pub replays:
        PreparedWorkflowTransitionReplays,
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowOperation<Schema, Operation, Input, Scope> {
    pub const fn required(&self) -> &RequiredWorkflowOperation {
        &self.required
    }

    pub fn into_required(self) -> RequiredWorkflowOperation {
        self.required
    }
}

/// A guarded operation the instance waits on: the operation and input it expects, and the
/// authority slot that lets exactly one matching effect run.
#[derive(Clone, Debug)]
pub struct RequiredWorkflowOperation {
    pub local_action: Option<super::super::local_actions::LocalActionDescriptor>,
    pub captured_input:
        Option<crate::domain_computation::primary_graph::WorkflowCapturedInputContract>,
    pub branch: crate::basis::WorthQueryProductBranch,
    pub instance: worth_relational::facade::identity::EntityId,
    pub original_instance: super::super::super::PublishedWorkflowInstanceRef,
    pub original_runtime: u64,
    pub original_program_revision: ApplicationProgramRevision,
    pub original_sources: Option<super::super::actor_context::OriginalActorSources>,
    pub node_path: String,
    pub transition_identity: String,
    pub transition_identity_bytes: [u8; 32],
    pub occurrence: u64,
    pub operation: String,
    pub input_type: String,
    pub binding: Option<String>,
    pub input_identity: [u8; 32],
    pub workflow_execution_requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
    pub authority: Arc<WorkflowOperationAuthoritySlot>,
}

#[derive(Debug)]
pub struct WorkflowOperationAuthoritySlot {
    pub authority: Mutex<Option<WorkflowOperationAuthority>>,
    pub issued: bool,
}

pub struct WorkflowOperationAuthority {
    pub captured_input:
        Option<crate::domain_computation::primary_graph::WorkflowCapturedInputContract>,
    pub operation:
        String,
    pub binding:
        String,
    pub transition_identity:
        [u8; 32],
    pub input_identity:
        [u8; 32],
    pub runtime_authority:
        u64,
    pub session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub observation:
        WorthQueryProductBranchReadIdentity,
    pub facts: Arc<[WorthQueryApplicationObservedFact]>,
    /// The instance's deadline; the operation commits only before it.
    pub deadline: Option<u64>,
    pub settlement_basis:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis,
    pub workflow_layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub approval_authority:
        Option<crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis>,
    pub workflow_execution_requirements:
        worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
    pub handle: WorthQueryPrimaryGraphIntegrationHandle,
    pub layout:
        Arc<crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout>,
}

impl std::fmt::Debug for WorkflowOperationAuthority {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkflowOperationAuthority")
            .field("observation", &self.observation)
            .field("fact_count", &self.facts.len())
            .finish_non_exhaustive()
    }
}

impl WorkflowOperationAuthoritySlot {
    #[doc(hidden)]
    pub fn take(&self) -> Option<WorkflowOperationAuthority> {
        self.authority
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    #[doc(hidden)]
    pub const fn was_issued(&self) -> bool {
        self.issued
    }
}

impl RequiredWorkflowOperation {
    /// Descriptive action kind; it carries no execution or settlement authority.
    pub const fn local_action_kind(&self) -> Option<super::super::WorkflowLocalActionKind> {
        match &self.local_action {
            Some(action) => Some(action.kind),
            None => None,
        }
    }
    pub fn workflow_execution_requirements(&self) -> &worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements{
        &self.workflow_execution_requirements
    }
    pub fn from_selected(
        branch: crate::basis::WorthQueryProductBranch,
        original_runtime: u64,
        original_program_revision: ApplicationProgramRevision,
        original_instance: super::super::super::PublishedWorkflowInstanceRef,
        original_sources: Option<super::super::actor_context::OriginalActorSources>,
        node_path: String,
        transition_identity: String,
        transition_identity_bytes: [u8; 32],
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowOperation,
        input_identity: [u8; 32],
        authority: Option<WorkflowOperationAuthority>,
    ) -> Self {
        Self {
            captured_input: None,
            local_action: None,
            branch,
            instance: original_instance.entity_id(),
            original_instance,
            original_runtime,
            original_program_revision,
            original_sources,
            node_path,
            transition_identity,
            transition_identity_bytes,
            occurrence,
            operation: selected.operation,
            input_type: selected.input_type,
            binding: selected.binding,
            workflow_execution_requirements: selected.workflow_execution_requirements,
            input_identity,
            authority: Arc::new(WorkflowOperationAuthoritySlot {
                issued: authority.is_some(),
                authority: Mutex::new(authority),
            }),
        }
    }

    pub const fn instance(&self) -> worth_relational::facade::identity::EntityId {
        self.instance
    }

    #[doc(hidden)]

    pub const fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }

    pub fn node_path(&self) -> &str {
        &self.node_path
    }

    pub fn transition_identity(&self) -> &str {
        &self.transition_identity
    }

    #[doc(hidden)]
    pub const fn transition_identity_bytes(&self) -> &[u8; 32] {
        &self.transition_identity_bytes
    }

    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn input_type(&self) -> &str {
        &self.input_type
    }

    pub fn binding(&self) -> Option<&str> {
        self.binding.as_deref()
    }

    #[doc(hidden)]
    pub const fn input_identity(&self) -> &[u8; 32] {
        &self.input_identity
    }

    #[doc(hidden)]
    pub fn authority_slot(&self) -> Arc<WorkflowOperationAuthoritySlot> {
        Arc::clone(&self.authority)
    }
}

impl WorkflowOperationAuthority {
    pub fn new(
        operation: String,
        binding: String,
        transition_identity: [u8; 32],
        input_identity: [u8; 32],
        runtime_authority: u64,
        session_identity:
            crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        observation: WorthQueryProductBranchReadIdentity,
        facts: Vec<WorthQueryApplicationObservedFact>,
        settlement_basis:
            crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis,
        workflow_layout:
            crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        approval_authority: Option<
            crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
        >,
        workflow_execution_requirements:
            worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<
            crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
        >,
    ) -> Self {
        Self {
            captured_input: None,
            operation,
            binding,
            transition_identity,
            input_identity,
            runtime_authority,
            session_identity,
            observation,
            facts: facts.into(),
            deadline: None,
            settlement_basis,
            workflow_layout,
            approval_authority,
            workflow_execution_requirements,
            handle,
            layout,
        }
    }

    pub fn with_deadline(
        mut self,
        deadline: Option<u64>,
    ) -> Self {
        self.deadline = deadline;
        self
    }

    #[doc(hidden)]
    pub fn validate_before_handler<Schema>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::ports::WorkflowOperationAuthorityRuntimePort<Schema>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        runtime.validate_workflow_operation_authority(
            phase,
            request,
            self.runtime_authority,
            &self.observation,
        )
    }

    #[doc(hidden)]

    pub fn facts(
        &self,
    ) -> &[WorthQueryApplicationObservedFact] {
        &self.facts
    }
}

impl RequiredWorkflowOperation {
    pub fn captured_input_contract(
        &self,
        _: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
    ) -> Option<&crate::domain_computation::primary_graph::WorkflowCapturedInputContract> {
        self.captured_input.as_ref()
    }
}
