//! Canonical commitment to descriptive persistent-name birth parts.
use super::PersistentNameIdentity;
use sha2::{Digest, Sha256};

pub const PERSISTENT_NAME_DOMAIN_TAG: &[u8] = b"worth.relational.persistent-name.v1\0";

pub fn canonical_birth_digest<'parent>(
    scheme_id: &str,
    occurrence_key: &str,
    role_path: &str,
    sorted_parent_digests: impl ExactSizeIterator<
        Item = &'parent [u8; PersistentNameIdentity::BYTE_LEN],
    >,
    canonical_discriminator: &[u8],
) -> [u8; PersistentNameIdentity::BYTE_LEN] {
    let mut hasher = Sha256::new();
    hasher.update(PERSISTENT_NAME_DOMAIN_TAG);
    write_len_prefixed(&mut hasher, scheme_id.as_bytes());
    write_len_prefixed(&mut hasher, occurrence_key.as_bytes());
    write_len_prefixed(&mut hasher, role_path.as_bytes());
    let parent_count = u64::try_from(sorted_parent_digests.len()).expect("parent count fits u64");
    hasher.update(parent_count.to_be_bytes());
    for parent in sorted_parent_digests {
        hasher.update(parent);
    }
    write_len_prefixed(&mut hasher, canonical_discriminator);
    hasher.finalize().into()
}

fn write_len_prefixed(hasher: &mut Sha256, bytes: &[u8]) {
    let len = u64::try_from(bytes.len()).expect("part length fits u64");
    hasher.update(len.to_be_bytes());
    hasher.update(bytes);
}
