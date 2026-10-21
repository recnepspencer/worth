mod captured_inputs;
mod facts;
mod identity;
mod observation;

pub(in crate::domain_computation::primary_graph) use facts::visit_workflow_proposal_facts;
pub(in crate::domain_computation::primary_graph) use identity::{
    derive_captured_workflow_proposal, derive_workflow_proposal_context_identity,
    WorkflowProposalCoverageMeaning, WorkflowProposalMeaning,
};
pub(in crate::domain_computation::primary_graph) use observation::{
    decode_identity, observe_captured_workflow_history, observe_captured_workflow_input,
    observe_workflow_operation_input, observe_workflow_proposal,
    observe_workflow_proposal_coverages,
};
