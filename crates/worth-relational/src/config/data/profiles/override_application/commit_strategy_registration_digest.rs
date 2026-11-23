use sha2::{Digest, Sha256};

use crate::commit_strategies::data::registration::CommitStrategyRegistration;

const REGISTRATION_SET_DIGEST_DOMAIN: &[u8] =
    b"worth.relational.config.commit_strategy_registration_set.v1";

pub(super) fn commit_strategy_registration_set_digest_hex(
    registrations: &[CommitStrategyRegistration],
) -> String {
    let digest = commit_strategy_registration_set_digest(registrations);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn commit_strategy_registration_set_digest(
    registrations: &[CommitStrategyRegistration],
) -> [u8; 32] {
    let mut descriptor_digests = registrations
        .iter()
        .map(|registration| registration.descriptor().digest().0)
        .collect::<Vec<_>>();
    descriptor_digests.sort();

    let mut canonical_bytes = Vec::new();
    encode_domain(&mut canonical_bytes, REGISTRATION_SET_DIGEST_DOMAIN);
    encode_u64(&mut canonical_bytes, descriptor_digests.len() as u64);
    for descriptor_digest in descriptor_digests {
        canonical_bytes.extend_from_slice(&descriptor_digest);
    }

    Sha256::digest(canonical_bytes).into()
}

fn encode_domain(bytes: &mut Vec<u8>, domain: &[u8]) {
    encode_u64(bytes, domain.len() as u64);
    bytes.extend_from_slice(domain);
}

fn encode_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
