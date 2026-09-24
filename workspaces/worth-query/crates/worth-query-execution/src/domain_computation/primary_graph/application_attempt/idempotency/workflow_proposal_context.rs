use super::{
    append_identity_slot as append_optional_identity_slot, WorthQueryApplicationIdempotencyBinding,
};

impl WorthQueryApplicationIdempotencyBinding {
    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_proposal_context(
        mut self,
        identity: &[u8; 32],
    ) -> Self {
        self.workflow_proposal_context_identity = Some(*identity);
        self
    }
}

pub(super) fn append_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-proposal-context", identity);
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryApplicationIdempotencyBinding;

    #[test]
    fn workflow_proposal_context_is_a_private_part_of_idempotency_intent() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let first = baseline.bind_workflow_proposal_context(&[3; 32]);
        let retry = baseline.bind_workflow_proposal_context(&[3; 32]);
        let drift = baseline.bind_workflow_proposal_context(&[4; 32]);

        assert_eq!(first.key_text(), retry.key_text());
        assert_eq!(first.intent_text(), retry.intent_text());
        assert_eq!(first.key_text(), drift.key_text());
        assert_ne!(first.intent_text(), drift.intent_text());
    }
}
