use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, RelationId};

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::workflow::instance::progression::{
    PreparedWorkflowProgressUpdate, WorkflowTransitionProgressBasis,
};

mod selection;
pub(in crate::domain_computation::primary_graph) use selection::{
    select_assessment_collection, select_current_transition, select_navigation_back_transition,
    select_proposal_replay_transition, select_proposal_transition,
    select_settled_replay_transition, select_terminal_transition, SelectedWorkflowApproval,
    SelectedWorkflowAssessment, SelectedWorkflowCondition, SelectedWorkflowOperation,
    SelectedWorkflowTransition, SelectedWorkflowTransitionKind,
};

pub(in crate::domain_computation::primary_graph) struct AdmittedWorkflowTransition<
    Schema,
    Operation,
    Input,
    Scope,
> {
    pub(super) read_set: WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    pub(super) instance: EntityId,
    pub(super) subject: EntityId,
    pub(super) node: EntityId,
    pub(super) node_path: String,
    pub(super) occurrence: u64,
    pub(super) live_membership: RelationId,
    pub(super) retire_live_membership: bool,
    pub(super) identity: String,
    pub(super) identity_bytes: [u8; 32],
    progress_basis: Option<WorkflowTransitionProgressBasis>,
}

impl<Schema, Operation, Input, Scope> AdmittedWorkflowTransition<Schema, Operation, Input, Scope> {
    pub(in crate::domain_computation::primary_graph) const fn instance(&self) -> EntityId {
        self.instance
    }

    pub(in crate::domain_computation::primary_graph) fn identity(&self) -> &str {
        &self.identity
    }

    pub(in crate::domain_computation::primary_graph) const fn identity_bytes(&self) -> &[u8; 32] {
        &self.identity_bytes
    }

    pub(in crate::domain_computation::primary_graph) fn node_path(&self) -> &str {
        &self.node_path
    }

    pub(in crate::domain_computation::primary_graph) const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    pub(in crate::domain_computation::primary_graph) fn subject(&self) -> EntityId {
        self.subject
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_progress_update(
        &self,
        outcome: ApplicationWorkflowControlOutcome,
        operation_receipt_identity: Option<[u8; 32]>,
    ) -> Result<PreparedWorkflowProgressUpdate, crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial>{
        self.progress_basis.as_ref().ok_or_else(|| {
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial::new(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
                "workflow transition progress basis is unavailable",
            )
        })?.prepare(
            self.node,
            self.occurrence,
            outcome,
            operation_receipt_identity,
            self.identity.clone(),
            self.identity_bytes,
            self.node_path.clone(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn is_assessment_collection(&self) -> bool {
        self.progress_basis
            .as_ref()
            .is_some_and(|basis| basis.progress().head() != self.node)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_assessment_collection_update(
        &self,
    ) -> Result<PreparedWorkflowProgressUpdate, crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial>{
        self.progress_basis.as_ref().ok_or_else(|| {
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial::new(
                crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
                "assessment collection progress basis is unavailable",
            )
        })?.prepare_assessment_collection(
            self.node,
            self.occurrence,
            self.identity.clone(),
            *self.identity_bytes(),
            self.node_path.clone(),
        )
    }

    pub(in crate::domain_computation::primary_graph) const fn read_set(
        &self,
    ) -> &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        &self.read_set
    }

    pub(in crate::domain_computation::primary_graph) fn into_read_set(
        self,
    ) -> WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        self.read_set
    }
}

pub(in crate::domain_computation::primary_graph) fn admit_workflow_transition<
    Schema,
    Operation,
    Input,
    Scope,
>(
    read_set: WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >,
    selected: SelectedWorkflowTransition,
    instance: EntityId,
    subject: EntityId,
    live_membership: RelationId,
    retire_live_membership: bool,
) -> AdmittedWorkflowTransition<Schema, Operation, Input, Scope> {
    AdmittedWorkflowTransition {
        read_set,
        instance,
        subject,
        node: selected.node,
        node_path: selected.node_path,
        occurrence: selected.occurrence,
        live_membership,
        retire_live_membership,
        identity: selected.identity,
        identity_bytes: selected.identity_bytes,
        progress_basis: selected.progress_basis,
    }
}
