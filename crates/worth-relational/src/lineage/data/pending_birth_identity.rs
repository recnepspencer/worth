//! The canonical persistent-name hash and pre-commit identity derivation.
//!
//! Minting, persisted-birth readmission, and pre-commit same-occurrence
//! discriminator lowering all derive identity bytes through this single door:
//! `(scheme_id, occurrence_key, role_path, sorted_parent_identities, canonical_discriminator)`
//! hashed with SHA-256 under [`PERSISTENT_NAME_DOMAIN_TAG`]. It never observes
//! native allocation coordinates (`RecordId`, `EntityId`, `PartitionId`,
//! `local_slot`, `generation`, `LineageId`, `CommitId`, or worker order).
//!
//! The returned identity proves nothing by itself. Commit finalization still
//! requires every referenced identity to be a live holder at the basis or a
//! birth it minted earlier in the same occurrence, and refuses otherwise.

use sha2::{Digest, Sha256};

use super::naming::PersistentNameIdentity;

/// Domain separator prefix for persistent name identity derivation.
pub const PERSISTENT_NAME_DOMAIN_TAG: &[u8] = b"worth.relational.persistent-name.v1\0";

impl PersistentNameIdentity {
    /// The identity commit finalization mints for a birth of exactly these
    /// parts. Parents may be given in any order.
    ///
    /// Complexity: O(parts bytes + P log P) for P parents.
    pub fn of_pending_birth(
        scheme_id: &str,
        occurrence_key: &str,
        role_path: &str,
        parent_identities: &[PersistentNameIdentity],
        canonical_discriminator: &[u8],
    ) -> Self {
        let mut sorted_parents = parent_identities.to_vec();
        sorted_parents.sort_unstable();
        Self::from_minted_digest(canonical_birth_digest(
            scheme_id,
            occurrence_key,
            role_path,
            sorted_parents.iter().map(PersistentNameIdentity::as_bytes),
            canonical_discriminator,
        ))
    }
}

/// The one canonical persistent-name hash: minting, persisted-birth
/// readmission, and pre-commit pending-birth derivation all derive identity
/// bytes only here.
///
/// Complexity: O(parts bytes + parents).
pub(crate) fn canonical_birth_digest<'parent>(
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
