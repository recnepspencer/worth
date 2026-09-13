//! Chunk identity cannot be constructed from a copied digest:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobChunkIdentity;
//! use worth_store_contracts::StableDigest;
//!
//! let digest = StableDigest::new("sha256:copied").unwrap();
//! let _forged = BlobChunkIdentity::from_integrity_parts(digest);
//! ```
//! Scoped chunks cannot be constructed from checksum-only evidence:
//! ```compile_fail
//! use worth_store_blob_chunks::ScopedBlobChunk;
//! use worth_store_physical_format::PhysicalChunkChecksumWitness;
//!
//! let checksum: PhysicalChunkChecksumWitness = todo!();
//! let _forged = ScopedBlobChunk::from_integrity_proof(checksum);
//! ```
//! Dedupe candidates cannot be constructed from digest-only evidence:
//! ```compile_fail
//! use worth_store_blob_chunks::BlobChunkDedupeCandidate;
//! use worth_store_contracts::StableDigest;
//!
//! let digest = StableDigest::new("sha256:copied").unwrap();
//! let _forged = BlobChunkDedupeCandidate::from_integrity_proof(digest);
//! ```
