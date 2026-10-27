use sha2::{Digest, Sha256};
use worth_foundational::facade::CanonicalDigestId;

pub fn digest_hash_parts(parts: &[String]) -> String {
    canonical_hash_parts(parts).render_hex()
}

pub fn canonical_hash_parts(parts: &[String]) -> CanonicalDigestId {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0x1f]);
    }
    CanonicalDigestId::new(hasher.finalize().into())
}

pub fn hash_parts(parts: &[String]) -> String {
    digest_hash_parts(parts)
}
