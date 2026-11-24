//! Canonical form of a birth claim, independently of readmission into a lineage.
use super::birth_digest::canonical_birth_digest;
use super::discriminator_scan::extract_embedded_name_refs;
use super::{PersistentNameIdentity, PersistentNameSubjectKind};

/// Raw 32 bytes of a persisted identity, before any readmission.
pub(crate) type PersistedNameBytes = [u8; PersistentNameIdentity::BYTE_LEN];

/// Every typed part of one persisted birth record, exactly as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersistedBirthClaim {
    pub(crate) stored_identity: PersistedNameBytes,
    pub(crate) subject_kind: PersistentNameSubjectKind,
    pub(crate) scheme_id: String,
    pub(crate) occurrence_key: String,
    pub(crate) role_path: String,
    pub(crate) parent_identities: Vec<PersistedNameBytes>,
    pub(crate) canonical_discriminator: Vec<u8>,
}

/// Why a persisted birth claim cannot be readmitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BirthReadmissionRefusal {
    /// Parents are not strictly ascending: unsorted or duplicated.
    NonCanonicalParents { stored: PersistedNameBytes },
    /// The discriminator fails the canonical discriminator scan.
    MalformedDiscriminator { stored: PersistedNameBytes },
    /// Re-hashing the typed parts does not reproduce the stored identity.
    DigestMismatch {
        stored: PersistedNameBytes,
        recomputed: PersistedNameBytes,
    },
    /// A parent is not an identity readmitted from a verified birth.
    UnverifiedParent {
        stored: PersistedNameBytes,
        parent: PersistedNameBytes,
    },
    /// A discriminator-embedded name is not a readmitted identity.
    UnverifiedDiscriminatorReference {
        stored: PersistedNameBytes,
        reference: PersistedNameBytes,
    },
}

pub(crate) fn verify_claim_encoding(
    claim: &PersistedBirthClaim,
) -> Result<Vec<PersistedNameBytes>, BirthReadmissionRefusal> {
    let stored = claim.stored_identity;
    let strictly_sorted = claim
        .parent_identities
        .windows(2)
        .all(|pair| pair[0] < pair[1]);
    if !strictly_sorted {
        return Err(BirthReadmissionRefusal::NonCanonicalParents { stored });
    }
    let embedded_references = extract_embedded_name_refs(&claim.canonical_discriminator)
        .map_err(|()| BirthReadmissionRefusal::MalformedDiscriminator { stored })?;
    let recomputed = canonical_birth_digest(
        &claim.scheme_id,
        &claim.occurrence_key,
        &claim.role_path,
        claim.parent_identities.iter(),
        &claim.canonical_discriminator,
    );
    if recomputed != stored {
        return Err(BirthReadmissionRefusal::DigestMismatch { stored, recomputed });
    }
    Ok(embedded_references)
}
