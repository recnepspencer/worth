use sha2::{Digest, Sha256};

pub(super) fn checkpoint_idempotency_material(seed: u64) -> [u8; 32] {
    digest_with_domain(b"worth.store.c8.checkpoint-schedule.v1", seed)
}

pub(super) fn dirty_material(seed: u64) -> [u8; 32] {
    digest_with_domain(b"worth.store.c8.dirty-mutation.v1", seed)
}

pub(super) fn no_effect_material(seed: u64) -> [u8; 32] {
    digest_with_domain(b"worth.store.c8.proven-no-effect.v1", seed)
}

pub(super) fn mutation_payload(material: [u8; 32]) -> Vec<u8> {
    payload_with_length(material, 8 * 1024)
}

pub(super) fn dirty_checkpoint_payload(material: [u8; 32], length: usize) -> Vec<u8> {
    payload_with_length(material, length)
}

fn payload_with_length(material: [u8; 32], length: usize) -> Vec<u8> {
    let mut payload = vec![0; length];
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte = material[index % material.len()]
            .wrapping_add(index as u8)
            .wrapping_mul(31);
    }
    payload
}

fn digest_with_domain(domain: &[u8], seed: u64) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(seed.to_le_bytes());
    digest.finalize().into()
}
