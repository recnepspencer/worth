//! Text encoding of an idempotency binding's key and intent.

use super::{WorthQueryIdempotencyEntityIdentity, WorthQueryIdempotencyScopeIdentity};

pub(super) fn append_identity_slot(encoded: &mut String, slot: &str, identity: Option<[u8; 32]>) {
    encoded.push(':');
    encoded.push_str(slot);
    encoded.push('=');
    match identity {
        Some(identity) => append_bytes(encoded, &identity),
        None => encoded.push('-'),
    }
}

pub(super) fn append_scope_slot(
    encoded: &mut String,
    identity: Option<WorthQueryIdempotencyScopeIdentity>,
) {
    encoded.push_str(":scope=");
    let Some(identity) = identity else {
        encoded.push('-');
        return;
    };
    append_bytes(encoded, &identity.package_identity);
    append_bytes(encoded, &identity.schema_identity);
    append_entity_identity(encoded, identity.principal);
    append_entity_identity(encoded, identity.scope);
}

fn append_entity_identity(encoded: &mut String, identity: WorthQueryIdempotencyEntityIdentity) {
    append_bytes(encoded, &identity.partition.to_be_bytes());
    append_bytes(encoded, &identity.local_slot.to_be_bytes());
    append_bytes(encoded, &identity.generation.to_be_bytes());
}

pub(super) fn encode_identity(identity: [u8; 32]) -> String {
    let mut encoded = String::with_capacity(64);
    append_bytes(&mut encoded, &identity);
    encoded
}

fn append_bytes(encoded: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
}

use super::{
    mutation_binding, workflow_definition, workflow_instance, workflow_proposal_context,
    workflow_transition, WorthQueryApplicationIdempotencyBinding,
};
impl WorthQueryApplicationIdempotencyBinding {
    /// Every intent slot, without the encoding's version; `intent_text` is
    /// what a key records.
    pub(super) fn intent_slots(self) -> String {
        let mut encoded = encode_identity(self.intent_identity);
        append_identity_slot(&mut encoded, "source", self.source_identity);
        append_identity_slot(&mut encoded, "operation", self.operation_identity);
        append_scope_slot(&mut encoded, self.operation_scope_identity);
        append_identity_slot(&mut encoded, "precondition", self.precondition_identity);
        append_identity_slot(&mut encoded, "input", self.governed_input_identity);
        append_identity_slot(&mut encoded, "proposal", self.governed_proposal_identity);
        append_identity_slot(
            &mut encoded,
            "conditional-definition",
            self.conditional_definition_identity,
        );
        workflow_definition::append_identity_slot(&mut encoded, self.workflow_definition_identity);
        workflow_instance::append_identity_slot(&mut encoded, self.workflow_instance_identity);
        workflow_proposal_context::append_identity_slot(&mut encoded, self.workflow_proposal);
        workflow_transition::append_identity_slot(&mut encoded, self.workflow_transition_identity);
        workflow_transition::append_support_identity_slot(
            &mut encoded,
            self.workflow_support_identity,
        );
        workflow_transition::append_client_key_identity_slot(
            &mut encoded,
            self.workflow_client_key_identity,
        );
        workflow_transition::append_approval_identity_slot(
            &mut encoded,
            self.workflow_approval_identity,
        );
        mutation_binding::append_identity_slot(&mut encoded, self.mutation_binding_identity);
        encoded
    }
}
