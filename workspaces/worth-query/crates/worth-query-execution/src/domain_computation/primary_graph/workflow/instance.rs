//! Authoritative branch-local workflow-instance facts.

mod facts;
mod progression;
mod state;
mod transition;

pub(in crate::domain_computation::primary_graph) use facts::visit_instance_start_facts;
pub use progression::WorthQueryWorkflowInstanceProgressCounters;
pub(in crate::domain_computation::primary_graph) use progression::{
    default_progress_retention_shards, PreparedWorkflowProgressUpdate,
    RetainedWorkflowInstanceProgressProjection, SettledWorkflowTransition,
    WorkflowInstanceProgress, WorkflowInstanceProgressKey, WorkflowInstanceProgressRetention,
    WorkflowInstanceProgressRetentionDenial, WorkflowTransitionLocator,
    WorkflowTransitionProgressBasis, WorkflowTransitionReplayProjection,
    WorkflowTransitionReplayRetention,
};
pub(in crate::domain_computation::primary_graph) use state::WorkflowInstanceState;
pub(in crate::domain_computation::primary_graph) use transition::{
    admit_workflow_transition, decode_transition_outcome, encode_transition_outcome,
    select_current_transition, select_proposal_replay_transition, select_proposal_transition,
    select_settled_replay_transition, select_terminal_transition, visit_terminal_transition_facts,
    visit_workflow_operation_transition_facts, visit_workflow_transition_facts,
    AdmittedWorkflowTransition, SelectedWorkflowApproval, SelectedWorkflowAssessment,
    SelectedWorkflowCondition, SelectedWorkflowOperation, SelectedWorkflowTransition,
    SelectedWorkflowTransitionKind,
};
