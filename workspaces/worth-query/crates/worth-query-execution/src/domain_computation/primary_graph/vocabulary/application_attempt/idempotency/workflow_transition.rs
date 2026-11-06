use super::{
    append_identity_slot as append_optional_identity_slot, WorthQueryApplicationIdempotencyBinding,
};
use sha2::{Digest, Sha256};

impl WorthQueryApplicationIdempotencyBinding {
    /// A distinct native commit namespace for accepting one original local effect.
    /// Original input, client key, source and preconditions remain in the intent.
    pub(in crate::domain_computation::primary_graph) fn bind_local_workflow_settlement(
        mut self,
        transition: &[u8; 32],
        receipt: &[u8; 32],
    ) -> Self {
        self.key_identity = local_settlement_key(transition);
        self.bind_workflow_transition(transition)
            .bind_workflow_operation(receipt)
    }
    pub(in crate::domain_computation::primary_graph) fn matches_local_workflow_settlement(
        &self,
        transition: &[u8; 32],
        receipt: &[u8; 32],
    ) -> bool {
        self.workflow_client_key_identity.is_some()
            && self.key_identity == local_settlement_key(transition)
            && self.workflow_transition_identity.as_ref() == Some(transition)
            && self.matches_workflow_operation(receipt)
    }
    pub(in crate::domain_computation::primary_graph) fn matches_captured_workflow_contract(
        &self,
        expected: &crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputContract,
    ) -> bool {
        self.workflow_client_key_identity == Some(expected.key)
            && self.source_identity == expected.source.map(|source| source.1)
            && self.source_partition_identity == expected.source.map(|source| source.2)
            && self.intent_identity == expected.input
    }
    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_support(
        mut self,
        identity: &[u8; 32],
    ) -> Self {
        self.workflow_support_identity = Some(*identity);
        self
    }

    pub const fn bind_workflow_transition(mut self, identity: &[u8; 32]) -> Self {
        self.workflow_transition_identity = Some(*identity);
        self
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_workflow_assessment(
        self,
        identity: &[u8; 32],
    ) -> Self {
        self.bind_workflow_support(identity)
    }

    pub const fn bind_workflow_condition(self, identity: &[u8; 32]) -> Self {
        self.bind_workflow_support(identity)
    }

    pub const fn bind_workflow_operation(self, identity: &[u8; 32]) -> Self {
        self.bind_workflow_support(identity)
    }

    pub fn bind_guarded_workflow_effect(mut self, transition: &[u8; 32]) -> Self {
        self.workflow_client_key_identity = Some(self.key_identity);
        self.key_identity = guarded_effect_key(transition);
        self.bind_workflow_operation(transition)
    }

    pub fn matches_guarded_workflow_effect(&self, transition: &[u8; 32]) -> bool {
        self.workflow_client_key_identity.is_some()
            && self.key_identity == guarded_effect_key(transition)
            && self.matches_workflow_operation(transition)
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

fn local_settlement_key(transition: &[u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:local-workflow-settlement-key:v1");
    digest.update(transition);
    digest.finalize().into()
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

fn guarded_effect_key(transition: &[u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:guarded-workflow-effect-key:v1");
    digest.update(transition);
    digest.finalize().into()
}

pub(super) fn append_client_key_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-client-key", identity);
    }
}

pub(super) fn append_approval_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "workflow-approval", identity);
    }
}
