use super::{
    append_identity_slot as append_optional_identity_slot, WorthQueryApplicationIdempotencyBinding,
};

impl WorthQueryApplicationIdempotencyBinding {
    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_support(
        mut self,
        identity: &[u8; 32],
    ) -> Self {
        self.workflow_support_identity = Some(*identity);
        self
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_transition(
        mut self,
        identity: &[u8; 32],
    ) -> Self {
        self.workflow_transition_identity = Some(*identity);
        self
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_assessment(
        self,
        identity: &[u8; 32],
    ) -> Self {
        self.bind_workflow_support(identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_condition(
        self,
        identity: &[u8; 32],
    ) -> Self {
        self.bind_workflow_support(identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_operation(
        self,
        identity: &[u8; 32],
    ) -> Self {
        self.bind_workflow_support(identity)
    }

    pub(in crate::domain_computation::primary_graph) fn matches_workflow_operation(
        &self,
        identity: &[u8; 32],
    ) -> bool {
        self.workflow_support_identity.as_ref() == Some(identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_approval(
        mut self,
        identity: &[u8; 32],
    ) -> Self {
        self.workflow_approval_identity = Some(*identity);
        self
    }
}

pub(super) fn append_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-transition", identity);
    }
}

pub(super) fn append_support_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-support", identity);
    }
}

pub(super) fn append_approval_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-approval", identity);
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryApplicationIdempotencyBinding;

    #[test]
    fn workflow_transition_is_a_private_part_of_idempotency_intent() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let first = baseline.bind_workflow_transition(&[3; 32]);
        let retry = baseline.bind_workflow_transition(&[3; 32]);
        let drift = baseline.bind_workflow_transition(&[4; 32]);

        assert_eq!(first.key_text(), retry.key_text());
        assert_eq!(first.intent_text(), retry.intent_text());
        assert_eq!(first.key_text(), drift.key_text());
        assert_ne!(first.intent_text(), drift.intent_text());
    }

    #[test]
    fn condition_source_is_a_private_part_of_idempotency_intent() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
            .bind_workflow_transition(&[3; 32]);
        let first = baseline.bind_workflow_condition(&[4; 32]);
        let retry = baseline.bind_workflow_condition(&[4; 32]);
        let drift = baseline.bind_workflow_condition(&[5; 32]);

        assert_eq!(first.intent_text(), retry.intent_text());
        assert_ne!(first.intent_text(), drift.intent_text());
    }
}
