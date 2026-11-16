//! Support-family payloads in the shared Query obligation envelope.
use super::*;
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
