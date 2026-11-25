//! The canonical persistent-name hash and pre-commit identity derivation.
//!
//! Minting, persisted-birth readmission, and pre-commit same-occurrence
//! discriminator lowering all derive identity bytes through this single door:
//! `(scheme_id, occurrence_key, role_path, sorted_parent_identities, canonical_discriminator)`
//! hashed with SHA-256 under the persistent-name domain tag. It never observes
//! native allocation coordinates (`RecordId`, `EntityId`, `PartitionId`,
//! `local_slot`, `generation`, `LineageId`, `CommitId`, or worker order).
//!
//! The returned identity proves nothing by itself. Commit finalization still
//! requires every referenced identity to be a live holder at the basis or a
//! birth it minted earlier in the same occurrence, and refuses otherwise.

use super::{
    birth_digest::canonical_birth_digest,
    naming::{PersistentNameIdentity, PersistentNameIdentityOwnerPort as _},
};

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
