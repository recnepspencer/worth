use worth_relational::facade::identity::{EntityId, RelationId};

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};

mod selection;
pub(in crate::domain_computation::primary_graph) use selection::{
    select_current_transition, select_proposal_replay_transition, select_proposal_transition,
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
    }
}
