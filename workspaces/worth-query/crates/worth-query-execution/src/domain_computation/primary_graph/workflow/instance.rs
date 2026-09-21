//! Authoritative branch-local workflow-instance facts.

mod facts;
mod state;
mod transition;

pub(in crate::domain_computation::primary_graph) use facts::visit_instance_start_facts;
pub(in crate::domain_computation::primary_graph) use state::WorkflowInstanceState;
pub(in crate::domain_computation::primary_graph) use transition::{
    admit_workflow_transition, decode_transition_outcome, encode_transition_outcome,
    select_current_transition, select_proposal_replay_transition, select_proposal_transition,
    select_settled_replay_transition, select_terminal_transition, visit_terminal_transition_facts,
    visit_workflow_transition_facts, AdmittedWorkflowTransition, SelectedWorkflowApproval,
    SelectedWorkflowAssessment, SelectedWorkflowTransition, SelectedWorkflowTransitionKind,
    SettledWorkflowTransition,
};
