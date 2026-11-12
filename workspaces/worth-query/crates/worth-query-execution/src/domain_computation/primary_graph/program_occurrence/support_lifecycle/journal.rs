//! Query owns the strict obligation history; Store transports opaque frames.
//!
//! Unlanded version-one format: each frame includes a nonzero local publication
//! generation. A resumed original keeps identical bytes. A new logical publish
//! after proven no-effect allocates another generation at the same next position.
//! Gaps are valid; committed generations strictly increase. This is correlation,
//! never obligation identity, a capability, or a serialized Home incarnation.
//! The live allocator is seeded only after cold fold and is not reset on failure.
//! On reopen, an uncompacted no-effect disappears; a compacted terminal belongs
//! to the prior physical namespace checkpoint generation. Actual physical key
//! issuance, not runtime identity, separates that prior equivalence class.
use std::collections::BTreeMap;
use worth_foundational::facade::{
    CanonicalDigestId, DurableCompositePublicationAttemptIdentity, HomeIdentity,
    ProgramSupportObligationIdentity, ProgramSupportObligationKind, ProgramSupportStatus,
};
mod codec;
mod generation;
mod ledger;
mod medium;
mod originals;
pub use generation::QueryProgramSupportPublicationGenerations;
pub use ledger::QueryProgramSupportLedger;
pub use medium::*;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod transfer_status_tests;

pub const QUERY_PROGRAM_SUPPORT_FRAME_MAX_BYTES: u64 = 214;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryProgramSupportObligation {
    pub identity: ProgramSupportObligationIdentity,
    pub kind: ProgramSupportObligationKind,
    pub source: CanonicalDigestId,
    pub target: CanonicalDigestId,
    /// The actual World attempt identity. Recorded DATA is never a live grant.
    pub original: DurableCompositePublicationAttemptIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryProgramSupportRecord {
    Install {
        revision: CanonicalDigestId,
    },
    Status {
        revision: CanonicalDigestId,
        before: ProgramSupportStatus,
        after: ProgramSupportStatus,
    },
    Acquire(QueryProgramSupportObligation),
    Release {
        identity: ProgramSupportObligationIdentity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryProgramSupportFrame {
    home: HomeIdentity,
    position: u64,
    predecessor: [u8; 32],
    generation: std::num::NonZeroU64,
    record: QueryProgramSupportRecord,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl QueryProgramSupportFrame {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn position(&self) -> u64 {
        self.position
    }
    pub const fn publication_generation(&self) -> std::num::NonZeroU64 {
        self.generation
    }
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn payload_digest(&self) -> [u8; 32] {
        use sha2::Digest;
        sha2::Sha256::digest(&self.bytes).into()
    }
    pub const fn home(&self) -> HomeIdentity {
        self.home
    }
    pub const fn record(&self) -> &QueryProgramSupportRecord {
        &self.record
    }
    pub fn readmit(bytes: &[u8]) -> Result<Self, QueryProgramSupportDenial> {
        codec::decode(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryProgramSupportDenial {
    UnsupportedVersion(u32),
    MalformedFrame,
    Checksum,
    ForeignHome,
    PrefixMismatch,
    IdentityExhausted,
    DuplicateProgram,
    UnrosteredProgram(CanonicalDigestId),
    StatusMismatch,
    IllegalStatusTransition,
    ObligationIdentityMismatch,
    ObligationSubjectMismatch,
    SupportUnavailable,
    UnknownObligation,
    DuplicateObligation,
    AdoptionReadmissionRequired,
    MissingOriginalInterpretationSource {
        identity: ProgramSupportObligationIdentity,
    },
    OriginalInterpretationProgramMismatch {
        identity: ProgramSupportObligationIdentity,
    },
    OwnerUnavailable,
    StaleIncarnation,
    PublicationPending,
    PublicationNoEffect,
    PublicationGenerationMismatch,
    PublicationUnresolved,
    Storage(String),
    Execution(worth_execution::WorkCeilingDenial),
}
