use sha2::{Digest, Sha256};

pub(crate) fn bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn value(domain: &str, value: &serde_json::Value) -> String {
    let mut digest = Sha256::new();
    digest.update((domain.len() as u64).to_le_bytes());
    digest.update(domain.as_bytes());
    // serde_json's default map is ordered; arrays preserve compiler order.
    let encoded = serde_json::to_vec(value).expect("JSON value serializes");
    digest.update((encoded.len() as u64).to_le_bytes());
    digest.update(encoded);
    format!("{:x}", digest.finalize())
}
