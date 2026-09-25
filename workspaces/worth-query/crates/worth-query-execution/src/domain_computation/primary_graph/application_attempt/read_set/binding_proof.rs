//! Private binding facts carried from the workflow owner to the effect owner.

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorkflowOperationBindingProof
{
    pub(in crate::domain_computation::primary_graph::application_attempt) binding: String,
    pub(in crate::domain_computation::primary_graph::application_attempt) transition_identity:
        [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_attempt) input_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_attempt) approval_authority:
        crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct MutationHandlerBindingProof
{
    pub(in crate::domain_computation::primary_graph::application_attempt) binding: &'static str,
    pub(in crate::domain_computation::primary_graph::application_attempt) input_identity: [u8; 32],
}
