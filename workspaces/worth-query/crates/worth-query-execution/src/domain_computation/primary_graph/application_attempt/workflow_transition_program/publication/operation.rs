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
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) admitted:
        crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) required:
        RequiredWorkflowOperation,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) program_revision:
        ApplicationProgramRevision,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) replays:
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
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) instance: worth_relational::facade::identity::EntityId,
    pub(super) node_path: String,
    pub(super) transition_identity: String,
    pub(super) transition_identity_bytes: [u8; 32],
    pub(super) occurrence: u64,
    pub(super) operation: String,
    pub(super) input_type: String,
    pub(super) binding: Option<String>,
    pub(super) input_identity: [u8; 32],
    pub(super) authority: Arc<WorkflowOperationAuthoritySlot>,
}

#[derive(Debug)]
pub struct WorkflowOperationAuthoritySlot {
    authority: Mutex<Option<WorkflowOperationAuthority>>,
    issued: bool,
}

pub struct WorkflowOperationAuthority {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) operation:
        String,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) binding:
        String,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) transition_identity:
        [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) input_identity:
        [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) runtime_authority:
        u64,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) session_identity:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) observation:
        WorthQueryProductBranchReadIdentity,
    pub(super) facts: Arc<[WorthQueryApplicationObservedFact]>,
    /// The instance's deadline; the operation commits only before it.
    pub(in crate::domain_computation::primary_graph::application_attempt) deadline: Option<u64>,
    pub(in crate::domain_computation::primary_graph::application_attempt) settlement_basis:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis,
    pub(in crate::domain_computation::primary_graph::application_attempt) workflow_layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) approval_authority:
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
    pub(super) handle: WorthQueryPrimaryGraphIntegrationHandle,
    pub(super) layout:
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
    pub(in crate::domain_computation::primary_graph) fn from_selected(
        branch: crate::basis::WorthQueryProductBranch,
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        transition_identity_bytes: [u8; 32],
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowOperation,
        input_identity: [u8; 32],
        authority: Option<WorkflowOperationAuthority>,
    ) -> Self {
        Self {
            branch,
            instance,
            node_path,
            transition_identity,
            transition_identity_bytes,
            occurrence,
            operation: selected.operation,
            input_type: selected.input_type,
            binding: selected.binding,
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
    pub(in crate::domain_computation::primary_graph) fn new(
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
        approval_authority:
            crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: Arc<
            crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
        >,
    ) -> Self {
        Self {
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
            handle,
            layout,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_deadline(
        mut self,
        deadline: Option<u64>,
    ) -> Self {
        self.deadline = deadline;
        self
    }

    #[doc(hidden)]
    pub fn validate_before_handler<Schema>(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
        };
        let mismatch = || {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "workflow operation authority is no longer current",
            )
        };
        if runtime.runtime.authority_identity().as_u64() != self.runtime_authority {
            return Err(mismatch());
        }
        check_request_live(request, "workflow operation authority")?;
        let selected = runtime
            .on_branch(self.observation.product_branch())
            .select()
            .map_err(|_| mismatch())?;
        let current =
            WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
        if !self.observation.same_branch_occurrence(&current) {
            return Err(mismatch());
        }
        let lease = WorthQueryApplicationSnapshotLease::acquire(
            self.handle.clone(),
            Arc::clone(&self.layout),
            selected.product().retained_clone(),
        )
        .map_err(|_| mismatch())?;
        runtime
            .readmit_workflow_approval_authority_on_product(
                &self.approval_authority,
                lease.product(),
                self.session_identity,
            )
            .map_err(|denial| {
                use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind;
                let kind = match denial.kind() {
                    WorthQueryOperationAuthorizationDenialKind::StalePrincipal => {
                        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalPrincipalStale
                    }
                    WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing => {
                        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable
                    }
                    WorthQueryOperationAuthorizationDenialKind::CapabilityExpired => {
                        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalExpired
                    }
                    WorthQueryOperationAuthorizationDenialKind::DelegationLineageChanged
                    | WorthQueryOperationAuthorizationDenialKind::DelegationRejected => {
                        WorthQueryApplicationAttemptDenialKind::WorkflowApprovalDelegationChanged
                    }
                    WorthQueryOperationAuthorizationDenialKind::TrustedTimeUnavailable => {
                        WorthQueryApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable
                    }
                    _ => WorthQueryApplicationAttemptDenialKind::WorkflowApprovalAuthorityDenied,
                };
                WorthQueryApplicationAttemptDenial::new(kind, denial.to_string())
            })?;
        lease.handle().with_runtime(|runtime| {
            let mut current = true;
            for fact in self.facts.iter() {
                check_request_live(request, "workflow operation authority")?;
                if !fact.remains_equal_in(runtime, lease.snapshot()) {
                    current = false;
                    break;
                }
            }
            if current {
                return Ok(());
            }
            // An instance that ended since is named before any fact it left
            // behind can mismatch.
            crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::instance_binding::deny_ended(
                runtime,
                lease.snapshot(),
                &self.workflow_layout,
                self.settlement_basis.instance(),
            )?;
            Err(mismatch())
        })
    }

    pub(in crate::domain_computation::primary_graph) fn facts(
        &self,
    ) -> &[WorthQueryApplicationObservedFact] {
        &self.facts
    }
}
