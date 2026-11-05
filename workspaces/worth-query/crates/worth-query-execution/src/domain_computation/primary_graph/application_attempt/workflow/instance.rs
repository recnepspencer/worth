//! Authoritative branch-local workflow-instance facts.

mod adoption_inventory;
pub(crate) mod facts;
pub(crate) mod migration;
pub(crate) mod progression;
pub(crate) mod start_provenance;
mod state;
mod transition;

pub(in crate::domain_computation::primary_graph) use adoption_inventory::{
    read_live_instances, WorkflowInventoriedTransition,
};
pub(in crate::domain_computation::primary_graph) use facts::{
    visit_instance_birth_facts, visit_instance_start_facts, WorkflowLineageCarry,
};
pub(in crate::domain_computation::primary_graph) use migration::{
    admit_workflow_migration, WorkflowPerformedEffect, WorkflowSuccession,
};

pub(in crate::domain_computation::primary_graph) use state::WorkflowInstanceState;
pub(in crate::domain_computation::primary_graph) use {
    progression::default_progress_retention_shards, progression::PreparedWorkflowProgressUpdate,
    progression::RetainedWorkflowInstanceProgressProjection,
    progression::SettledWorkflowTransition, progression::WorkflowAssessmentEvidenceLocator,
    progression::WorkflowInstanceProgress, progression::WorkflowInstanceProgressRetention,
    progression::WorkflowInstanceProgressRetentionDenial, progression::WorkflowTransitionLocator,
    progression::WorkflowTransitionProgressBasis,
    progression::WorkflowTransitionProgressObservation,
    progression::WorkflowTransitionReplayProjection,
    progression::WorkflowTransitionReplayRetention,
};
pub(in crate::domain_computation::primary_graph) use {
    transition::admit_workflow_transition, transition::decode_transition_outcome,
    transition::encode_transition_outcome, transition::select_assessment_collection,
    transition::select_back_action_transition, transition::select_current_proposal_need,
    transition::select_current_transition, transition::select_navigation_back_transition,
    transition::select_proposal_replay_transition, transition::select_proposal_transition,
    transition::select_settled_replay_transition, transition::select_terminal_transition,
    transition::select_termination_action_transition, transition::visit_terminal_transition_facts,
    transition::visit_workflow_operation_settlement_facts,
    transition::visit_workflow_transition_facts, transition::AdmittedWorkflowTransition,
    transition::SelectedWorkflowApproval, transition::SelectedWorkflowAssessment,
    transition::SelectedWorkflowInbound, transition::SelectedWorkflowOperation,
    transition::SelectedWorkflowProposal, transition::SelectedWorkflowTransition,
    transition::SelectedWorkflowTransitionKind, transition::WorkflowOperationSettlementBasis,
};
