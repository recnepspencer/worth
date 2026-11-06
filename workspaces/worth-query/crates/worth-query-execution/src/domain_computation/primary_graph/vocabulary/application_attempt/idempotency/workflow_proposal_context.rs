use super::{
    append_identity_slot as append_optional_identity_slot, WorthQueryApplicationIdempotencyBinding,
};

/// Proposal replay binds its instance and, only for the captured profile, its
/// exact target inventory. Empty captured meaning preserves the old encoding.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct WorkflowProposalIdempotency {
    context: Option<[u8; 32]>,
    inputs: Option<[u8; 32]>,
}
impl WorkflowProposalIdempotency {
    pub(super) const fn empty() -> Self {
        Self {
            context: None,
            inputs: None,
        }
    }
}

impl WorthQueryApplicationIdempotencyBinding {
    pub const fn bind_workflow_proposal_context(mut self, identity: &[u8; 32]) -> Self {
        self.workflow_proposal.context = Some(*identity);
        self
    }
    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_captured_inputs(
        mut self,
        identity: Option<[u8; 32]>,
    ) -> Self {
        self.workflow_proposal.inputs = identity;
        self
    }
}

pub(super) fn append_identity_slot(encoded: &mut String, proposal: WorkflowProposalIdempotency) {
    if proposal.context.is_some() {
        append_optional_identity_slot(encoded, "workflow-proposal-context", proposal.context);
    }
    if proposal.inputs.is_some() {
        append_optional_identity_slot(encoded, "workflow-captured-inputs", proposal.inputs);
    }
}
