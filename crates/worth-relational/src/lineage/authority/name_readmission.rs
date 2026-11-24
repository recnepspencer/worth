//! Readmission of persisted persistent-name births across restarts.
//!
//! A [`PersistentNameIdentity`] is never deserialized. Persisted bytes regain
//! an identity only here, in two steps:
//!
//! 1. [`verify_persisted_birth_digest`] checks that the claimed parents are
//!    strictly sorted, that the discriminator passes the canonical scan, and
//!    that re-hashing every typed birth part with the one canonical
//!    persistent-name hash reproduces the stored 32 bytes exactly.
//! 2. [`VerifiedBirthDigest::bind_references`] then requires every parent
//!    and every discriminator-embedded name to be an identity that was itself
//!    readmitted from a verified birth.
//!
//! The two steps are separate so a commit's births can all be verified before
//! any of them is required to reference another one born in the same commit.

use crate::lineage::data::{
    PersistentNameBirthParts, PersistentNameBirthRecord, PersistentNameIdentity,
};
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameBirthPartsOwnerPort as _;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameBirthRecordOwnerPort as _;
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameIdentityOwnerPort as _;

use crate::lineage::data::birth_wire::verify_claim_encoding;
pub(crate) use crate::lineage::data::birth_wire::{
    BirthReadmissionRefusal, PersistedBirthClaim, PersistedNameBytes,
};

/// Proof that a claim's typed parts re-hash to its stored identity; its
/// references are not yet bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerifiedBirthDigest {
    identity: PersistentNameIdentity,
    claim: PersistedBirthClaim,
    embedded_references: Vec<PersistedNameBytes>,
}

/// Verifies one persisted birth's canonical form and digest.
///
/// Complexity: O(parents + discriminator bytes + parts bytes).
pub(crate) fn verify_persisted_birth_digest(
    claim: PersistedBirthClaim,
) -> Result<VerifiedBirthDigest, BirthReadmissionRefusal> {
    let embedded_references = verify_claim_encoding(&claim)?;
    Ok(VerifiedBirthDigest {
        identity: PersistentNameIdentity::from_minted_digest(claim.stored_identity),
        claim,
        embedded_references,
    })
}

impl VerifiedBirthDigest {
    /// The identity re-derived from the verified typed parts.
    pub(crate) const fn identity(&self) -> PersistentNameIdentity {
        self.identity
    }

    /// Binds parents and embedded names to readmitted identities.
    ///
    /// `readmitted` answers whether raw bytes name an identity already
    /// readmitted from a verified birth. Complexity: O((parents + embedded
    /// names) * lookup).
    pub(crate) fn bind_references(
        self,
        readmitted: impl Fn(&PersistedNameBytes) -> Option<PersistentNameIdentity>,
    ) -> Result<PersistentNameBirthRecord, BirthReadmissionRefusal> {
        let stored = self.claim.stored_identity;
        let mut parents = Vec::with_capacity(self.claim.parent_identities.len());
        for parent in &self.claim.parent_identities {
            let identity = readmitted(parent).ok_or(BirthReadmissionRefusal::UnverifiedParent {
                stored,
                parent: *parent,
            })?;
            parents.push(identity);
        }
        if let Some(reference) = self
            .embedded_references
            .iter()
            .find(|reference| readmitted(reference).is_none())
        {
            return Err(BirthReadmissionRefusal::UnverifiedDiscriminatorReference {
                stored,
                reference: *reference,
            });
        }
        let claim = self.claim;
        let parts = PersistentNameBirthParts::new(
            claim.scheme_id,
            claim.occurrence_key,
            claim.role_path,
            parents,
            claim.canonical_discriminator,
        );
        Ok(PersistentNameBirthRecord::new(
            self.identity,
            claim.subject_kind,
            parts,
        ))
    }
}
