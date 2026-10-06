//! Bank-facing read-only view of Query's committed dispatch outbox.

use worth_query_host::facade::primary_graph::{
    WorthQueryCommittedDispatchOutboxObservation,
    WorthQueryCommittedDispatchOutboxReadDenial as QueryDenial,
    WorthQueryExternalDispatchAttemptDenial as QueryAttemptDenial,
};

use crate::{BankCommitReceipt, BankIdentityRuntime};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankCommittedDispatchOutboxObservation {
    query: WorthQueryCommittedDispatchOutboxObservation,
}

impl BankCommittedDispatchOutboxObservation {
    pub fn correlation(&self) -> &[u8; 32] {
        self.query.record().correlation().bytes()
    }

    pub fn correlation_family(&self) -> &str {
        self.query.record().correlation_family().as_str()
    }

    pub fn effect(&self) -> &str {
        self.query.record().effect()
    }

    pub const fn protocol_identity(&self) -> &worth_foundational::facade::BoundaryProtocolIdentity {
        self.query.record().protocol_identity()
    }

    pub const fn protocol_version(&self) -> worth_foundational::facade::BoundaryProtocolVersion {
        self.query.record().protocol_version()
    }

    pub const fn maximum_payload_bytes(&self) -> u64 {
        self.query.record().maximum_payload_bytes()
    }

    pub fn payload(&self) -> &[u8] {
        self.query.record().payload()
    }

    pub const fn outcome_identity(&self) -> u64 {
        self.query.record().outcome_identity()
    }

    pub const fn commit_id(&self) -> u64 {
        self.query.commit_reference().commit_id.0
    }

    pub const fn commit_version(&self) -> u64 {
        self.query.commit_reference().version_id.0
    }

    pub fn commit_branch(&self) -> &str {
        &self.query.commit_reference().branch_id.0
    }
}

/// Why Bank could not read a committed effect's outbox row. Each Query cause
/// keeps its own variant, because a settling publication and a commit whose
/// exact version is gone ask the caller for different next steps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankCommittedDispatchOutboxReadDenial {
    ForeignRuntime,
    Missing,
    /// More than one committed effect claims the correlation.
    AmbiguousCorrelation,
    /// The effect's commit is still publishing.
    PendingPublication,
    /// The effect committed, but its completed-commit evidence is not yet
    /// indexed.
    CommittedIndexUnavailable,
    WrongRecordKind,
    NotAuthoritative,
    /// The exact committed version or its retained basis is no longer
    /// available; no later read can recover it.
    ExactCommitUnavailable,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
    Malformed,
    CommitMismatch,
    RecordMismatch,
}

impl BankIdentityRuntime {
    /// Reads a committed effect from a fresh Query-provider owner view.
    pub fn observe_committed_dispatch_outbox(
        &self,
        receipt: &BankCommitReceipt,
    ) -> Result<Option<BankCommittedDispatchOutboxObservation>, BankCommittedDispatchOutboxReadDenial>
    {
        receipt
            .recovery_evidence()
            .observe_dispatch_outbox(self.application_runtime())
            .map(|observation| {
                observation.map(|query| BankCommittedDispatchOutboxObservation { query })
            })
            .map_err(Into::into)
    }
}

impl From<QueryDenial> for BankCommittedDispatchOutboxReadDenial {
    fn from(denial: QueryDenial) -> Self {
        match denial {
            QueryDenial::ForeignRuntime => Self::ForeignRuntime,
            QueryDenial::Missing => Self::Missing,
            QueryDenial::AmbiguousCorrelation => Self::AmbiguousCorrelation,
            QueryDenial::PendingPublication => Self::PendingPublication,
            QueryDenial::CommittedIndexUnavailable => Self::CommittedIndexUnavailable,
            QueryDenial::WrongRecordKind => Self::WrongRecordKind,
            QueryDenial::NotAuthoritative => Self::NotAuthoritative,
            QueryDenial::ExactCommitUnavailable => Self::ExactCommitUnavailable,
            QueryDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            QueryDenial::SnapshotIdentityExhausted => Self::SnapshotIdentityExhausted,
            QueryDenial::Malformed => Self::Malformed,
            QueryDenial::CommitMismatch => Self::CommitMismatch,
            QueryDenial::RecordMismatch => Self::RecordMismatch,
        }
    }
}

/// Why Query could not admit one physical dispatch attempt. Each cause keeps
/// its own variant, because a full in-flight window clears on its own while a
/// foreign or mismatched original never does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankExternalDispatchAttemptDenial {
    ForeignRelationalRuntime,
    ForeignProductWorld,
    PublicationCommitMismatch,
    /// An inbound-completing effect carries no co-committed operation slot.
    InboundOperationSlotMissing,
    /// The runtime has no physical attempt identities left.
    AttemptIdentityExhausted,
    OutstandingDispatchMissing,
    /// The original dispatch's publication has not settled yet.
    OriginalPublicationPending,
    OutstandingDispatchMismatch,
    /// The concurrent physical sends for this effect are at their installed
    /// limit; one frees when an in-flight send settles.
    InFlightCapacityExhausted,
}

impl From<QueryAttemptDenial> for BankExternalDispatchAttemptDenial {
    fn from(denial: QueryAttemptDenial) -> Self {
        match denial {
            QueryAttemptDenial::ForeignRelationalRuntime => Self::ForeignRelationalRuntime,
            QueryAttemptDenial::ForeignProductWorld => Self::ForeignProductWorld,
            QueryAttemptDenial::PublicationCommitMismatch => Self::PublicationCommitMismatch,
            QueryAttemptDenial::InboundOperationSlotMissing => Self::InboundOperationSlotMissing,
            QueryAttemptDenial::AttemptIdentityExhausted => Self::AttemptIdentityExhausted,
            QueryAttemptDenial::OutstandingDispatchMissing => Self::OutstandingDispatchMissing,
            QueryAttemptDenial::OriginalPublicationPending => Self::OriginalPublicationPending,
            QueryAttemptDenial::OutstandingDispatchMismatch => Self::OutstandingDispatchMismatch,
            QueryAttemptDenial::InFlightCapacityExhausted => Self::InFlightCapacityExhausted,
        }
    }
}
