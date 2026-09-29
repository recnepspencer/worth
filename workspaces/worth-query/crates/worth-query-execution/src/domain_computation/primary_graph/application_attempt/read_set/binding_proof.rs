//! Private binding facts carried from the workflow owner to the effect owner.

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorkflowOperationBindingProof
{
    pub(in crate::domain_computation::primary_graph::application_attempt) binding: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) transition_identity:
        [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_attempt) input_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_attempt) approval_authority:
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
    pub(in crate::domain_computation::primary_graph::application_attempt) settlement_basis:
        crate::domain_computation::primary_graph::workflow::instance::WorkflowOperationSettlementBasis,
    pub(in crate::domain_computation::primary_graph::application_attempt) workflow_layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct MutationHandlerBindingProof
{
    pub(in crate::domain_computation::primary_graph::application_attempt) binding: &'static str,
    pub(in crate::domain_computation::primary_graph::application_attempt) input_identity: [u8; 32],
}

impl MutationHandlerBindingProof {
    /// Refuses a commit whose idempotency binding is not for the request the
    /// handler decided on: it must name the handler's own mutation binding and
    /// derive its intent from the input the handler decided on.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn refusal(
        &self,
        idempotency: &super::super::WorthQueryApplicationIdempotencyBinding,
    ) -> Option<super::super::WorthQueryApplicationCommitDenial> {
        if !idempotency.is_for_mutation_binding(self.binding) {
            return Some(
                super::super::WorthQueryApplicationCommitDenial::mutation_binding_mismatch(),
            );
        }
        (*idempotency.intent_identity() != self.input_identity)
            .then(super::super::WorthQueryApplicationCommitDenial::mutation_input_mismatch)
    }
}
