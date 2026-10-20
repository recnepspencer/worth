//! Receipt-backed origin of native settlement; it never claims to have executed a body again.
use super::{RequiredWorkflowOperation, ValidatedWorkflowOperationReceipt};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationCommitDenial, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture;
pub(in crate::domain_computation::primary_graph::application_attempt) struct LocalActionSettlementProof
{
    binding: String,
    input: [u8; 32],
    transition: [u8; 32],
    receipt: [u8; 32],
}
impl LocalActionSettlementProof {
    pub(super) fn from_performed(
        required: &RequiredWorkflowOperation,
        performed: &ValidatedWorkflowOperationReceipt<'_>,
    ) -> Self {
        Self {
            binding: required
                .binding()
                .expect("validated local action binding")
                .to_owned(),
            input: *required.input_identity(),
            transition: *required.transition_identity_bytes(),
            receipt: performed.identity(),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn refusal(
        &self,
        idempotency: &WorthQueryApplicationIdempotencyBinding,
        posture: WorthQueryApplicationEffectPosture,
        has_execution_proof: bool,
    ) -> Option<WorthQueryApplicationCommitDenial> {
        let valid = posture == WorthQueryApplicationEffectPosture::Platform
            && !has_execution_proof
            && idempotency.is_for_mutation_binding(&self.binding)
            && idempotency.intent_identity() == &self.input
            && idempotency.matches_local_workflow_settlement(&self.transition, &self.receipt);
        (!valid).then(|| {
            WorthQueryApplicationCommitDenial::workflow_settlement_denied(
                &WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                    "native local settlement differs from its performed body origin",
                ),
            )
        })
    }
}
