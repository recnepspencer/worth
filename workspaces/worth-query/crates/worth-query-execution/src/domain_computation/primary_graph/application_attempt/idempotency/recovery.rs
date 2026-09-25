use super::WorthQueryApplicationIdempotencyBinding;

impl WorthQueryApplicationIdempotencyBinding {
    pub(in crate::domain_computation::primary_graph) fn matches_recovery_request(
        &self,
        request: &Self,
    ) -> bool {
        self.key_identity == request.key_identity
            && self.workflow_client_key_identity == request.workflow_client_key_identity
            && self.intent_identity == request.intent_identity
            && self.source_identity == request.source_identity
            && self.workflow_transition_identity == request.workflow_transition_identity
            && self.workflow_support_identity == request.workflow_support_identity
    }
}
