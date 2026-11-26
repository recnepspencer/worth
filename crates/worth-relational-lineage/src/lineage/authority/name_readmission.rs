//! Readmission of persisted persistent-name births across restarts.
//!
//! A [`crate::contracts::lineage::data::naming::PersistentNameIdentity`] is
//! never trusted from bytes: Journal replay re-verifies every persisted birth
//! by checking that its claimed parents are strictly sorted, that its
//! discriminator passes the canonical scan, and that re-hashing every typed
//! birth part with the one canonical persistent-name hash reproduces the
//! stored 32 bytes exactly.

use crate::contracts::lineage::data::birth_wire::verify_claim_encoding;
use crate::contracts::lineage::data::birth_wire::BirthReadmissionRefusal;
pub use crate::contracts::lineage::data::birth_wire::PersistedBirthClaim;

/// Verifies one persisted birth's canonical form and digest.
///
/// Complexity: O(parents + discriminator bytes + parts bytes).
pub fn verify_persisted_birth_digest(
    claim: PersistedBirthClaim,
) -> Result<(), BirthReadmissionRefusal> {
    verify_claim_encoding(&claim).map(|_| ())
}
