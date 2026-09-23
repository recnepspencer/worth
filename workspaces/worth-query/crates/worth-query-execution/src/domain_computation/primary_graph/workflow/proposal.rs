mod facts;
mod identity;
mod observation;

pub(in crate::domain_computation::primary_graph) use facts::visit_workflow_proposal_facts;
pub(in crate::domain_computation::primary_graph) use identity::{
    derive_workflow_proposal, derive_workflow_proposal_context_identity, WorkflowProposalMeaning,
};
pub(in crate::domain_computation::primary_graph) use observation::{
    observe_workflow_operation_input, observe_workflow_proposal,
};
