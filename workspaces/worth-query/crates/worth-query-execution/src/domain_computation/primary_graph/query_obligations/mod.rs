//! Query owns the strict obligation history; Store transports opaque frames.
//!
//! Current version-two format with explicit family routing: each frame includes a nonzero local publication
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
mod program_support;
pub use generation::QueryObligationPublicationGenerations;
pub use ledger::QueryObligationLedger;
pub use medium::*;
pub use program_support::{QueryProgramSupportObligation, QueryProgramSupportRecord};

pub const QUERY_OBLIGATION_FRAME_MAX_BYTES: u64 = 16 * 1024;
pub(crate) mod history;
mod managed_cleanup;
pub use managed_cleanup::{
    QueryManagedCleanupObligation, QueryManagedCleanupOriginal, QueryManagedCleanupRecord,
    QueryManagedCleanupRelease,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryObligationRecord {
    ProgramSupport(QueryProgramSupportRecord),
    ManagedCleanup(QueryManagedCleanupRecord),
}
impl From<QueryProgramSupportRecord> for QueryObligationRecord {
    fn from(record: QueryProgramSupportRecord) -> Self {
        Self::ProgramSupport(record)
    }
}
impl From<QueryManagedCleanupRecord> for QueryObligationRecord {
    fn from(record: QueryManagedCleanupRecord) -> Self {
        Self::ManagedCleanup(record)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryObligationFrame {
    home: HomeIdentity,
    position: u64,
    predecessor: [u8; 32],
    generation: std::num::NonZeroU64,
    record: QueryObligationRecord,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl QueryObligationFrame {
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
    pub const fn record(&self) -> &QueryObligationRecord {
        &self.record
    }
    pub fn readmit(bytes: &[u8]) -> Result<Self, QueryObligationDenial> {
        codec::decode(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryObligationDenial {
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
    CleanupReleaseUnrecorded {
        identity: worth_foundational::facade::ManagedCleanupObligationIdentity,
    },
    CleanupInconsistent {
        identity: worth_foundational::facade::ManagedCleanupObligationIdentity,
    },
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
