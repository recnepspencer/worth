//! Semantic admission to release one published blob generation.
//!
//! Physical reachability, age, and encoded reclaim records cannot issue this
//! proof. The production issuer belongs to a successor retention authority.

#[derive(Debug, PartialEq, Eq)]
pub struct AdmittedBlobReleaseProof {
    store: [u8; 16],
    object: [u8; 16],
    generation: u64,
    publication_allocation_epoch: [u8; 16],
    publication_record_ordinal: u64,
    publication_frame_sha256: [u8; 32],
    issuer_evidence_sha256: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReleaseProofDenial {
    InvalidIdentity,
    InvalidGeneration,
    MissingEvidence,
}

impl AdmittedBlobReleaseProof {
    pub const fn store(&self) -> [u8; 16] {
        self.store
    }

    pub const fn object(&self) -> [u8; 16] {
        self.object
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn publication_frame_sha256(&self) -> [u8; 32] {
        self.publication_frame_sha256
    }

    pub const fn publication_allocation_epoch(&self) -> [u8; 16] {
        self.publication_allocation_epoch
    }

    pub const fn publication_record_ordinal(&self) -> u64 {
        self.publication_record_ordinal
    }

    pub const fn issuer_evidence_sha256(&self) -> [u8; 32] {
        self.issuer_evidence_sha256
    }

    /// Certification exercises the consumer before a production issuer ships.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_admit(
        store: [u8; 16],
        object: [u8; 16],
        generation: u64,
        publication_allocation_epoch: [u8; 16],
        publication_record_ordinal: u64,
        publication_frame_sha256: [u8; 32],
        issuer_evidence_sha256: [u8; 32],
    ) -> Result<Self, BlobReleaseProofDenial> {
        if store == [0; 16]
            || object == [0; 16]
            || publication_allocation_epoch == [0; 16]
            || publication_record_ordinal == 0
        {
            return Err(BlobReleaseProofDenial::InvalidIdentity);
        }
        if generation == 0 {
            return Err(BlobReleaseProofDenial::InvalidGeneration);
        }
        if publication_frame_sha256 == [0; 32] || issuer_evidence_sha256 == [0; 32] {
            return Err(BlobReleaseProofDenial::MissingEvidence);
        }
        Ok(Self {
            store,
            object,
            generation,
            publication_allocation_epoch,
            publication_record_ordinal,
            publication_frame_sha256,
            issuer_evidence_sha256,
        })
    }
}
