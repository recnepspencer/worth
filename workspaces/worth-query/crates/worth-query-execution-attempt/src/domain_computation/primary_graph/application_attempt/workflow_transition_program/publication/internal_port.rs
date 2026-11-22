//! Crate-boundary internal ports for workflow operation/condition requirements
//! and authority slots. Never exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::condition::RequiredWorkflowCondition;
use super::operation::{
    RequiredWorkflowOperation, WorkflowOperationAuthority, WorkflowOperationAuthoritySlot,
};
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationObservedFact, WorthQueryPrimaryGraphIntegrationHandle,
};

pub trait WorkflowOperationInternalPort: Sized {
    #[allow(clippy::too_many_arguments)]
    fn from_selected(
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
    ) -> Self;

    fn local_action_ref(&self) -> Option<&super::super::local_actions::LocalActionDescriptor>;
    fn set_local_action(
        &mut self,
        action: Option<super::super::local_actions::LocalActionDescriptor>,
    );
    fn captured_input_ref(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorkflowCapturedInputContract>;
    fn set_captured_input(
        &mut self,
        captured: Option<crate::domain_computation::primary_graph::WorkflowCapturedInputContract>,
    );
    fn authority_slot_ref(&self) -> &Arc<WorkflowOperationAuthoritySlot>;
    fn set_authority_slot(&mut self, slot: Arc<WorkflowOperationAuthoritySlot>);
}

impl WorkflowOperationInternalPort for RequiredWorkflowOperation {
    fn from_selected(
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
        Self::from_selected(
            branch,
            original_runtime,
            original_program_revision,
            original_instance,
            original_sources,
            node_path,
            transition_identity,
            transition_identity_bytes,
            occurrence,
            selected,
            input_identity,
            authority,
        )
    }

    fn local_action_ref(&self) -> Option<&super::super::local_actions::LocalActionDescriptor> {
        self.local_action.as_ref()
    }

    fn set_local_action(
        &mut self,
        action: Option<super::super::local_actions::LocalActionDescriptor>,
    ) {
        self.local_action = action;
    }

    fn captured_input_ref(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorkflowCapturedInputContract> {
        self.captured_input.as_ref()
    }

    fn set_captured_input(
        &mut self,
        captured: Option<crate::domain_computation::primary_graph::WorkflowCapturedInputContract>,
    ) {
        self.captured_input = captured;
    }

    fn authority_slot_ref(&self) -> &Arc<WorkflowOperationAuthoritySlot> {
        &self.authority
    }

    fn set_authority_slot(&mut self, slot: Arc<WorkflowOperationAuthoritySlot>) {
        self.authority = slot;
    }
}

pub trait WorkflowOperationAuthorityInternalPort: Sized {
    #[allow(clippy::too_many_arguments)]
    fn from_parts(
        operation: String,
        binding: String,
        transition_identity: [u8; 32],
        input_identity: [u8; 32],
        runtime_authority: u64,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        observation: WorthQueryProductBranchReadIdentity,
        facts: Vec<WorthQueryApplicationObservedFact>,
        settlement_basis: crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis,
        workflow_layout: crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        approval_authority: Option<
            crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
        >,
        workflow_execution_requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<
            crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
        >,
    ) -> Self;

    fn with_deadline(self, deadline: Option<u64>) -> Self;
    fn captured_input_ref(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorkflowCapturedInputContract>;
    fn set_captured_input(
        &mut self,
        captured: Option<crate::domain_computation::primary_graph::WorkflowCapturedInputContract>,
    );
    fn operation_ref(&self) -> &str;
    fn binding_ref(&self) -> &str;
    fn transition_identity_bytes(&self) -> [u8; 32];
    fn input_identity_bytes(&self) -> [u8; 32];
    fn runtime_authority_value(&self) -> u64;
    fn session_identity_value(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
    fn observation_ref(&self) -> &WorthQueryProductBranchReadIdentity;
    fn deadline_value(&self) -> Option<u64>;
    fn settlement_basis_ref(
        &self,
    ) -> &crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis;
    fn workflow_layout_ref(
        &self,
    ) -> &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
    fn approval_authority_ref(
        &self,
    ) -> Option<&crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis>;
    fn workflow_execution_requirements_ref(
        &self,
    ) -> &worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements;
    fn handle_ref(&self) -> &WorthQueryPrimaryGraphIntegrationHandle;
    fn layout_ref(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout>;
}

impl WorkflowOperationAuthorityInternalPort for WorkflowOperationAuthority {
    fn from_parts(
        operation: String,
        binding: String,
        transition_identity: [u8; 32],
        input_identity: [u8; 32],
        runtime_authority: u64,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        observation: WorthQueryProductBranchReadIdentity,
        facts: Vec<WorthQueryApplicationObservedFact>,
        settlement_basis: crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis,
        workflow_layout: crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
        approval_authority: Option<
            crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
        >,
        workflow_execution_requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements,
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<
            crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
        >,
    ) -> Self {
        Self::new(
            operation,
            binding,
            transition_identity,
            input_identity,
            runtime_authority,
            session_identity,
            observation,
            facts,
            settlement_basis,
            workflow_layout,
            approval_authority,
            workflow_execution_requirements,
            handle,
            layout,
        )
    }

    fn with_deadline(self, deadline: Option<u64>) -> Self {
        Self::with_deadline(self, deadline)
    }

    fn captured_input_ref(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorkflowCapturedInputContract> {
        self.captured_input.as_ref()
    }

    fn set_captured_input(
        &mut self,
        captured: Option<crate::domain_computation::primary_graph::WorkflowCapturedInputContract>,
    ) {
        self.captured_input = captured;
    }

    fn operation_ref(&self) -> &str {
        &self.operation
    }

    fn binding_ref(&self) -> &str {
        &self.binding
    }

    fn transition_identity_bytes(&self) -> [u8; 32] {
        self.transition_identity
    }

    fn input_identity_bytes(&self) -> [u8; 32] {
        self.input_identity
    }

    fn runtime_authority_value(&self) -> u64 {
        self.runtime_authority
    }

    fn session_identity_value(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.session_identity
    }

    fn observation_ref(&self) -> &WorthQueryProductBranchReadIdentity {
        &self.observation
    }

    fn deadline_value(&self) -> Option<u64> {
        self.deadline
    }

    fn settlement_basis_ref(
        &self,
    ) -> &crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis
{
        &self.settlement_basis
    }

    fn workflow_layout_ref(
        &self,
    ) -> &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout {
        &self.workflow_layout
    }

    fn approval_authority_ref(
        &self,
    ) -> Option<&crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis>
    {
        self.approval_authority.as_ref()
    }

    fn workflow_execution_requirements_ref(
        &self,
    ) -> &worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements
{
        &self.workflow_execution_requirements
    }

    fn handle_ref(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.handle
    }

    fn layout_ref(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout>
    {
        &self.layout
    }
}

pub trait WorkflowConditionInternalPort: Sized {
    fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowCondition,
    ) -> Self;

    fn condition_ref(
        &self,
    ) -> &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowCondition;
}

impl WorkflowConditionInternalPort for RequiredWorkflowCondition {
    fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowCondition,
    ) -> Self {
        Self::from_selected(
            instance,
            node_path,
            transition_identity,
            occurrence,
            selected,
        )
    }

    fn condition_ref(
        &self,
    ) -> &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowCondition
    {
        &self.condition
    }
}
