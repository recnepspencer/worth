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
    append_bytes(encoded, &identity.runtime_authority.to_be_bytes());
    append_bytes(encoded, &identity.binding_runtime.to_be_bytes());
    append_bytes(encoded, &identity.binding_generation.to_be_bytes());
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
