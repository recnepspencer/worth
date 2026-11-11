//! Query's positioned correction journal DATA and its trusted Home transport.
//! Semantic material, disclosure and retention-pin admission remain Query-owned;
//! decoding this envelope never constructs an executable original or a receipt.
mod framing;
mod medium;
pub use medium::*;
use std::num::NonZeroU64;
use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolVersion, DurableArtifactRetention, HomeIdentity,
    RetentionClass,
};
pub const QUERY_CORRECTION_JOURNAL_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.query.corrections");
pub const QUERY_CORRECTION_JOURNAL_VERSION: BoundaryProtocolVersion =
    BoundaryProtocolVersion::new(1);

/// A checksummed envelope is DATA, not semantic material or publication authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryCorrectionMaterialFrame {
    home: HomeIdentity,
    position: NonZeroU64,
    predecessor: [u8; 32],
    generation: NonZeroU64,
    body_retention: RetentionClass,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
// This class covers the journal's ordering and correlation metadata. The body
// carries its own owner's class; the transport cannot reclaim or interpret it.
impl DurableArtifactRetention for QueryCorrectionMaterialFrame {
    const RETENTION_CLASS: RetentionClass = RetentionClass::Authoritative;
}
impl QueryCorrectionMaterialFrame {
    pub fn prepare(
        home: HomeIdentity,
        position: NonZeroU64,
        predecessor: [u8; 32],
        generation: NonZeroU64,
        body_retention: RetentionClass,
        body: &[u8],
    ) -> Result<Self, QueryCorrectionMaterialDenial> {
        framing::encode(
            home,
            position,
            predecessor,
            generation,
            body_retention,
            body,
        )
    }
    /// Own the actual bytes. Declared lengths can only select within this slice.
    pub fn readmit(bytes: Vec<u8>) -> Result<Self, QueryCorrectionMaterialDenial> {
        framing::decode(bytes)
    }
    pub const fn home(&self) -> HomeIdentity {
        self.home
    }
    pub const fn position(&self) -> u64 {
        self.position.get()
    }
    pub const fn predecessor(&self) -> [u8; 32] {
        self.predecessor
    }
    pub const fn publication_generation(&self) -> NonZeroU64 {
        self.generation
    }
    pub const fn body_retention_class(&self) -> RetentionClass {
        self.body_retention
    }
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn payload_digest(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        Sha256::digest(&self.bytes).into()
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn body(&self) -> &[u8] {
        &self.bytes[framing::HEADER..self.bytes.len() - 32]
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryCorrectionMaterialDenial {
    UnsupportedVersion(u32),
    MalformedFrame,
    Checksum,
    Allocation,
    ForeignHome,
    StaleIncarnation,
    PrefixMismatch,
    IdentityExhausted,
    OwnerUnavailable,
    AdmissionRequired,
    AdmissionAlreadyRecorded,
    Unavailable,
    NonPortableValue,
    RetentionClassMismatch,
    OriginalMismatch,
    NativeSourceUnavailable,
    NativeRecovery(Box<worth_relational::facade::durability::RelationalRecoveryStop>),
    MaterialContentUnavailable,
    OriginalBindingUnavailable,
    ProgramUnavailable,
    MaterialAlreadyCaptured,
    MaterialNotReady,
    MaterialPinned,
    RetainedMaterialExpired,
    UnknownPin,
    PreparationDischarged,
    PinOwnerMismatch,
    PinAlreadyClaimed,
    PinIntentDrift,
    TargetChanged,
    PublicationPending,
    PublicationUnresolved,
    PublicationNoEffect,
    Storage(String),
    Execution(worth_execution::WorkCeilingDenial),
}
#[cfg(test)]
mod tests;
