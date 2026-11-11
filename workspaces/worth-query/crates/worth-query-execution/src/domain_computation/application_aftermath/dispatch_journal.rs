//! Query owns the strict external-dispatch custody history; the Home transports opaque frames.
//!
//! Unlanded version-one format. An attempt is recorded durably before its
//! external contact and is named by the journal position that recorded it, so
//! no process-local runtime or ordinal enters durable dispatch identity. Each
//! attempt receives exactly one recorded observation. Proven no effect is its
//! own observation; it is never unresolved, compensation or completion.
//!
//! Each frame includes a nonzero local publication generation: a resumed
//! original keeps identical bytes, while a new logical publish after proven
//! no-effect allocates another generation at the same next position. The
//! generation is correlation only; it is never attempt identity or authority.
use std::collections::BTreeMap;
use worth_foundational::facade::{
    CanonicalDigestId, DispatchAttemptIdentity, HomeIdentity, InboundSourceControl,
};
mod codec;
mod custody;
mod generation;
mod inbound;
mod ledger;
mod medium;
mod read_limits;
pub use custody::WorthQueryDispatchJournal;
pub(in crate::domain_computation) use custody::{
    QueryInboundJournalAdmission, QueryInboundSourceControlObservation,
};
use generation::QueryDispatchPublicationGenerations;
pub(in crate::domain_computation) use inbound::{
    QueryInboundAcceptedMessage, QueryInboundDispatchSource, QueryInboundMessageKey,
    QueryInboundRecord, QueryInboundRecordedMessage, QueryTransportBinding,
};
pub use ledger::QueryDispatchCustody;
use ledger::QueryDispatchLedger;
pub use medium::{QueryDispatchMedium, QueryDispatchPublicationOutcome};
pub(in crate::domain_computation) use worth_proof::QueryDispatchCloseCustody;
#[cfg(test)]
mod tests;

pub const QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES: u64 = 149;

pub use worth_foundational::facade::ExternalEffectNoEffectCause;

/// The attempt contacted the external owner and its effect remains unknown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryDispatchUnresolvedCause {
    Timeout,
    Disconnect,
    LostResponse,
    DuplicatedAcknowledgement,
    UnsupportedProtocolVersion,
    UnknownProviderOutcome,
    ObservationDerivationDenied,
}

/// The one durable observation of an attempt's external contact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryDispatchRecordedObservation {
    Completed,
    Acknowledged,
    ProvenNoEffect(ExternalEffectNoEffectCause),
    Unresolved(QueryDispatchUnresolvedCause),
}

/// Recorded DATA naming the co-committed outbox effect an attempt contacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct QueryDispatchAttempt {
    identity: DispatchAttemptIdentity,
    correlation: CanonicalDigestId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryDispatchRecord {
    Inbound(QueryInboundRecord),
    SourceControl {
        source: CanonicalDigestId,
        posture: InboundSourceControl,
    },
    /// Recorded before the external call; never a dispatch capability.
    Attempt(QueryDispatchAttempt),
    Observation {
        attempt: DispatchAttemptIdentity,
        observation: QueryDispatchRecordedObservation,
    },
}

/// A requested record. An attempt's identity is assigned by the position that records it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryDispatchRecordIntent {
    Inbound(QueryInboundRecord),
    SourceControl {
        source: CanonicalDigestId,
        posture: InboundSourceControl,
    },
    Attempt {
        correlation: CanonicalDigestId,
    },
    Observation {
        attempt: DispatchAttemptIdentity,
        observation: QueryDispatchRecordedObservation,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryDispatchFrame {
    home: HomeIdentity,
    position: u64,
    predecessor: [u8; 32],
    generation: std::num::NonZeroU64,
    record: QueryDispatchRecord,
    bytes: Vec<u8>,
    digest: [u8; 32],
}
impl QueryDispatchFrame {
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
    const fn record(&self) -> &QueryDispatchRecord {
        &self.record
    }
    pub fn readmit(
        bytes: &[u8],
        maximum_frame_bytes: u64,
    ) -> Result<Self, QueryDispatchJournalDenial> {
        codec::decode(bytes, maximum_frame_bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryDispatchJournalDenial {
    UnsupportedVersion(u32),
    MalformedFrame,
    Checksum,
    ForeignHome,
    PrefixMismatch,
    IdentityExhausted,
    AttemptIdentityMismatch,
    /// The same effect already has an attempt awaiting its observation.
    AttemptAwaitingObservation,
    /// A completed external effect is never contacted again.
    EffectAlreadyCompleted,
    UnknownAttempt,
    AttemptAlreadyObserved,
    /// The recorded history ends inside an external contact with `effect`.
    /// Graceful reopen never guesses that fate; crash recovery owns it.
    UnobservedAttempt {
        attempt: DispatchAttemptIdentity,
        effect: CanonicalDigestId,
    },
    /// This runtime has no durable Home journal; its custody is process-local.
    JournalUnavailable,
    OwnerUnavailable,
    StaleIncarnation,
    PublicationPending,
    PublicationNoEffect,
    PublicationGenerationMismatch,
    PublicationUnresolved,
    Storage(String),
    Execution(worth_execution::WorkCeilingDenial),
}
